// StepFlow example that walks through the following Steps:
// 1 - first name + last name
// 2 - email
// 3 - email validation
// 4 - success


use std::convert::From;
use std::{collections::{HashMap}};
use std::sync::{Arc, RwLock};
use warp::{Filter, Rejection, Reply};
use tracing_attributes::instrument;
use tera::{Context, Tera};

use stepflow::{data::StringValue, object::{ObjectStore, IdError}};
use stepflow::data::{StateData, InvalidValue, VarId, TrueValue};
use stepflow::step::StepId;
use stepflow::action::ActionId;
use stepflow::{AdvanceBlockedOn, Session, SessionId, Error};

mod helpers;
use helpers::{StepInfo, VarInfo, VarType, register_steps, register_vars, register_actions, ActionInfo};

#[derive(Debug)]
struct WarpError(Error);
impl warp::reject::Reject for WarpError {}

#[derive(Debug)]
struct SerdeJsonError(serde_json::Error);
impl warp::reject::Reject for SerdeJsonError {}

#[derive(Debug)]
struct TeraError(tera::Error);
impl warp::reject::Reject for TeraError {}

const SESSION_ROOT_PATH: &str = "register";
const TERA_TEMPLATE_PATH: &str = "examples/warp/tera-templates/**/*";

fn register_all_steps(session: &mut Session, varnames: &Vec<&'static str>) -> Result<(), Error> {
    let stepinfos = vec![
        StepInfo("root", None, varnames.clone()),   // root step expects all the fields as output
        StepInfo("name", None, vec!["first_name", "last_name"]),
        StepInfo("email", None, vec!["email"]),
        StepInfo("email_validated", Some(vec!["email"]), vec!["email_validated"]),
        StepInfo("success_validated", None, vec!["success_validated"]),
    ];
    let step_ids = register_steps(session, stepinfos)?;

    // add steps to root
    let root_step_id = step_ids.get(0).unwrap();
    let root_step = session.step_store_mut().get_mut(&root_step_id).unwrap();
    for step_id in step_ids.get(1..) {
        root_step.push_substep(step_id[0])
    }

    // add root to session
    session.push_root_substep(root_step_id.clone());

    Ok(())
}

fn register_all_actions(session: &mut Session) -> Result<Vec<ActionId>, Error> {
    let email_validated_var = session.var_store().get_by_name("email_validated").unwrap().clone();
    let mut email_validated_statedata = StateData::new();
    email_validated_statedata.insert(email_validated_var, TrueValue::new().boxed()).unwrap();

    let success_validated_var = session.var_store().get_by_name("success_validated").unwrap().clone();
    let mut success_validated_statedata = StateData::new();
    success_validated_statedata.insert(success_validated_var, TrueValue::new().boxed()).unwrap();

    let actionsinfos = vec![
        ActionInfo::UriAction { step_name: None, base_path: format!("/{}/{}", SESSION_ROOT_PATH, session.id())},
        ActionInfo::SetDataAction { step_name: Some("email_validated"), statedata: email_validated_statedata, after_attempt: 2},
        ActionInfo::SetDataAction { step_name: Some("success_validated"), statedata: success_validated_statedata, after_attempt: 1},
    ];
    register_actions(session, actionsinfos)
}

fn create_tera_contexts() -> HashMap<&'static str, Context> {
    // add Tera contexts
    let mut stepid_to_context: HashMap<&str, Context> = HashMap::new();

    let mut name_context = Context::new();
    name_context.insert("template_file", "name.html");
    name_context.insert("title", "name");
    name_context.insert("first_name_field", "first_name");
    name_context.insert("last_name_field", "last_name");
    stepid_to_context.insert("name", name_context);

    let mut email_context = Context::new();
    email_context.insert("template_file", "email.html");
    email_context.insert("title", "email");
    stepid_to_context.insert("email", email_context);

    let mut email_valid_context = Context::new();
    email_valid_context.insert("template_file", "email_valid.html");
    email_valid_context.insert("title", "email valid");
    stepid_to_context.insert("email_validated", email_valid_context.clone());

    let mut success_context = Context::new();
    success_context.insert("template_file", "success.html");
    success_context.insert("title", "success!");
    stepid_to_context.insert("success_validated", success_context.clone());

    stepid_to_context
}

// put together vars, steps and actions to create a new session
#[instrument]
fn new_session(session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>) -> Result<SessionId, Error> {
    // create a session
    let mut session_store = session_store.write().unwrap();
    let session_id = session_store
        .insert_new(|session_id| Ok(Session::new(session_id)))
        .map_err(|err| Error::from(err))?;
    let mut session = session_store.get_mut(&session_id).ok_or_else(|| Error::SessionId(IdError::IdMissing(session_id)))?;

    // register Vars
    let varinfos = vec![
        VarInfo("first_name", VarType::String),
        VarInfo("last_name", VarType::String),
        VarInfo("email", VarType::Email),
        VarInfo("email_validated", VarType::True),
        VarInfo("success_validated", VarType::True),
    ];
    register_vars(&mut session, &varinfos)?;

    // register steps
    let varnames = varinfos.iter().map(|v| v.0).collect();
    register_all_steps(&mut session, &varnames)?;

    // register actions
    register_all_actions(&mut session)?;

    Ok(session_id)
}

fn redirect_as_other(uri: &str) -> impl Reply {
    warp::reply::with_header(
        warp::http::StatusCode::SEE_OTHER,
        warp::http::header::LOCATION,
        uri.to_string(),
    )
}

fn redirect_from_advance(advance_result: AdvanceBlockedOn, session_id: &SessionId) -> Result<impl Reply, Error> {
    match advance_result {
        AdvanceBlockedOn::ActionStartWith(_, val) => {
            if let Some(uri) = val.downcast::<StringValue>() {
                Ok(redirect_as_other(uri.val()))
            } else {
                Err(Error::Other)
            }
        }
        AdvanceBlockedOn::ActionCannotFulfill => {
            Err(Error::Other)
        }
        AdvanceBlockedOn::FinishedAdvancing => {
            let done_uri = format!("/done/{}", session_id);
            Ok(redirect_as_other(&done_uri[..]))
        }
    }
}

#[instrument]
fn advance(session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>, session_id: &SessionId, step_output: Option<(&StepId, StateData)>) -> Result<AdvanceBlockedOn, Error> {
    let mut session_store_write = session_store.write().unwrap();
    let session = session_store_write.get_mut(&session_id).unwrap();
    session.advance(step_output)
}

pub async fn new_handler<'a>(session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>) -> Result<impl Reply, Rejection> {
    let session_id = new_session(session_store.clone()).unwrap();
    let advance_result = advance(session_store, &session_id, None)
        .map_err(|e| warp::reject::custom(WarpError(e)))?;
    redirect_from_advance(advance_result, &session_id)
        .map_err(|e| warp::reject::custom(WarpError(e)))
}

#[instrument(skip(templates))]
pub async fn step_handler(
        session_id: SessionId,
        step_name: String,
        session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>,
        templates: Arc<HashMap<&str, Context>>,
        error: Option<&Error>)
    -> Result<impl Reply, Rejection>
{
    let session_store_read = session_store.read().unwrap();
    let session = session_store_read.get(&session_id).unwrap();
    let tera = Tera::new(TERA_TEMPLATE_PATH).map_err(|e| warp::reject::custom(TeraError(e)))?;
    let base_template: &Context = templates.get(&step_name[..]).ok_or_else(|| warp::reject::reject())?;
    let mut template = base_template.clone();
    
    if let Some(error) = error {
        template.insert("error", error);
        if let Error::InvalidVars(invalid) = error {
            let name_to_error = invalid.0.iter()
                .filter_map(|(var_id, val_invalid)| {
                    let name = session.var_store().name_from_id(var_id)?;
                    Some((name.clone(), *val_invalid))
                })
                .collect::<HashMap<&str, InvalidValue>>();
            template.insert("field_errors", &name_to_error);
        }
    }

    let template_name = template.get("template_file").map(|v| v.as_str().unwrap()).ok_or_else(|| warp::reject::reject())?;
    let render = tera.render(&template_name.to_string()[..], &template).map_err(|e| warp::reject::custom(TeraError(e)))?;
    Ok(warp::reply::html(render))
}

#[instrument]
pub async fn post_step_handler(
        session_id: SessionId,
        step_name: String,
        session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>,
        form_data: HashMap<String, String>,
        templates: Arc<HashMap<&str, Context>>)
        -> Result<Box<dyn Reply>, Rejection> {

    let mut field_errors: HashMap<VarId, InvalidValue> = HashMap::new();
    let state_data;
    let step_id;
    {
        // get the session
        let session_store_read = session_store.read().unwrap();
        let session = session_store_read.get(&session_id).unwrap();

        // convert form strings to Vars
        let state_vals = form_data
            .into_iter()
            .filter_map(|(field_name, val)| {
                let var = session.var_store().get_by_name(&field_name)?;
                let value_result = var.value_from_str(&val[..]);
                match value_result {
                    Ok(value) => Some((var, value)),
                    Err(e) => {
                        field_errors.insert(var.id().clone(), e);
                        None
                    },
                }
            });

        // create state data with Vars
        state_data = StateData::from_vals(state_vals).map_err(|e| Error::InvalidVars(e));

        // grab the StepId
        step_id = session.step_store().id_from_name(&step_name[..]).unwrap().clone();
    }

    // get the warp reply
    let reply = state_data
        .and_then(|output_data| advance(session_store.clone(), &session_id, Some((&step_id, output_data))))
        .and_then(|advance_result| redirect_from_advance(advance_result, &session_id))
        .map(|r| Box::new(r) as _);    

    // if there are errors, display the form again with the error info
    match reply {
        Ok(r) if field_errors.len() == 0 => Ok(r),
        Ok(_) => {
            let error = Error::InvalidVars(stepflow_data::InvalidVars::new(field_errors));
            step_handler(session_id, step_name, session_store, templates, Some(&error))
                .await
                .map(|r| Box::new(r) as _)
        },
        Err(error) => {
            step_handler(session_id, step_name, session_store, templates, Some(&error))
                .await
                .map(|r| Box::new(r) as _)
        }
    }
}

pub async fn done_handler(session_id: SessionId, session_store: Arc<RwLock<ObjectStore<Session, SessionId>>>) -> Result<impl Reply, Rejection> {
    // dump the data when we're all done
    let session_store_read = session_store.read().unwrap();
    let session = session_store_read.get(&session_id).unwrap();
    let json = serde_json::to_string(session.state_data()).map_err(|e| warp::reject::custom(SerdeJsonError(e)))?;
    Ok(json)
}

pub async fn home_handler() -> Result<impl Reply, Rejection> {
    let tera = Tera::new(TERA_TEMPLATE_PATH).map_err(|e| warp::reject::custom(TeraError(e)))?;
    let mut template = tera::Context::new();
    template.insert("start_path", SESSION_ROOT_PATH);
    let render = tera.render("home.html", &template).map_err(|e| warp::reject::custom(TeraError(e)))?;
    Ok(warp::reply::html(render))
}

fn with_session_store_rc(session_store_rc: Arc<RwLock<ObjectStore<Session, SessionId>>>) -> impl Filter<Extract = (Arc<RwLock<ObjectStore<Session, SessionId>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || session_store_rc.clone())
}

fn with_templates(templates: Arc<HashMap<&str, Context>>) -> impl Filter<Extract = (Arc<HashMap<&str, Context>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || templates.clone())
}


fn with_none<T>() -> impl Filter<Extract = (Option<T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(|| None)
}

#[tokio::main]
async fn main() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "debug,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let session_store = ObjectStore::new();
    let session_store_rc = Arc::new(RwLock::new(session_store));
    let templates = create_tera_contexts();
    let templates_rc = Arc::new(templates);

    // paths for routes
    let register_path = warp::path(SESSION_ROOT_PATH);
    let step_path = 
        register_path
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::path::end());

    // route to create the session
    let new_route =
        register_path
        .and(warp::path::end())
        .and(warp::get())
        .and(with_session_store_rc(session_store_rc.clone()))
        .and_then(new_handler);

    // route to show a step
    let step_route = 
        step_path.clone()
        .and(warp::get())
        .and(with_session_store_rc(session_store_rc.clone()))
        .and(with_templates(templates_rc.clone()))
        .and(with_none())
        .and_then(step_handler);

    // route to handle a step posting
    let step_route_post = 
        step_path.clone()
        .and(warp::post())
        .and(with_session_store_rc(session_store_rc.clone()))
        .and(warp::body::form())
        .and(with_templates(templates_rc.clone()))
        .and_then(post_step_handler);

    // route when the session is done
    let session_done_route = warp::path("done")
        .and(warp::path::param())
        .and(with_session_store_rc(session_store_rc.clone()))
        .and_then(done_handler);

    // all the session routes together
    let routes = new_route
        .or(step_route)
        .or(step_route_post)
        .or(session_done_route);

    // and the home route
    let home_route = 
        warp::path::end()
        .and(warp::get())
        .and_then(home_handler);

    println!("Server started on port 8080");
    warp::serve(home_route.or(routes).with(warp::trace::request()))
        .run(([0, 0, 0, 0], 8080))
        .await;
}
