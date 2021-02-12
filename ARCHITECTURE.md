# StepFlow Architecture
Quick and dirty instructions + basic architecture of StepFlow. 

NOTE: A *lot* of work needed on this.

## Usage Quick Summary
Read the docs (TBD) for detailed info. 

A `Session` is the main object that's used. 
- Register a series of `Step`s with `Session.step_store`
- Add those steps to the root step with `Session.push_root_substep`. All sub-steps to a `Step` get evaluated in order by default.
- Call `Session.advance` to progress the flow. It does two things:
    - try to move to the next step (based on having the proper outputs to exit the current step and the proper inputs to enter the next step). 
    - Irregardless of success or failure to advance to the next step, an `Action` is executed to try and fulfill the outputs of the current step. If it can be done without blocking (as noted by the return value `ActionResult::Finished`), `Session.advance` will continue to run advancing as far as possible until a blocking `ActionResult` is returned by an action.
- Data is contained in the `Session.state_data` and is kept up-to-date based from `Action` results. It is composed of a series of `Var`s and validated `Value`s.

## Overall
The overall goal of the system is to fulfill a declared set of outputs. It enables the fulfillment of the outputs in a user-friendly manner by:
- Allowing different types of actions to obtain those outputs. These actions are designed to either show a user interface to request data from the user or to obtain them from other sources, such as databases, interruption-free.
- Breaking down the actions and outputs into smaller sub-steps. This can be more user-friendly in addition to making it easier to re-use steps in other flows.
- Defining a common data layer to allow different parts of the flow to integrate with different systems.


## Components

### `stepflow-test-util`
- Utilities to make testing easier. Should only be a dev dependency.

### `stepflow-base`
- `ObjectStore` is the general store for various object types in StepFlow. It is used to maintain scoped object IDs and optional unique friendly names. Objects IDs are effectively a weak reference to the object.
- IDs in the system are created with the macro `generate_id_type`

### `stepflow-data`
- `StateData` is a typed data store for session data. Within it, a `Var` is the type of data and the `Value` is the data itself. The actual value is wrapped in a `ValidVal` which serves to confirm that the value has been validated via `Var.validate_val_type`
- `Var`s are simple declarations of a `Value`. Their existence allows a step to define which values are required for its inputs and outputs without the data itself.
- `Value` is the typed data used in StepFlow. They store their data in a mostly fixed set of `BaseValue` types. They can add an extra layer of validation (i.e. `EmailValue`) for higher level types
- Both vars and values support downcasting via `[Var|Value].downcast<T>` and `[Var|Value].is<T>`.

### `stepflow-step`
- `Step` is a single step in a user flow. It is defined by the input vars required to enter the step, the output vars it generates and any sub-steps that are contained.
- By default, sub-steps are executed in-order

### `stepflow-action`
- `Action`s operate on a step to try fulfilling the outputs of that step.
- If an `Action` is blocking because of required user input, long running task or any other reason, it returns either an `ActionResult::CannotFulfill` or an `ActionResult::StartWith`.
- `ActionResult::StartWith` may return a value to the implementor to complete the action. For example, if the action is to direct the user to an HTML form, it may return a `ActionResult::StartWith("/step/5")` to tell the implementor to redirect the user to that URL which contains the form.
- `ActionResult::Finished` indicates fulfillment of the outputs and allows the flow to continue without user interruption.

### `stepflow-session`
- `Session` can be thought of as 2 main components. 
    - First is the definition of the user flow. This is effectively the group of `ObjectStore`s (`Session.step_store`, `Session.action_store` and `Session.var_store`) along with declaring initial Steps to execute via `Session.push_root_substep`.
    - Second is the execution of a session, of which `Session.advance` is the primary entry point.
- `Session.advance` does two things:
    - First, it tries to advance to the next step using its internal `Session.step_id_dfs` instance.
    - Next it tries to run an action on the current step. If there's an Action specifically assigned to that step via `Session.set_action_for_step` it will execute first. If not, or the previous action could not fulfill, the global action (set with `Session.set_action_for_step` with no step) will start.
- `Session.step_id_dfs` is a DFS iterator on the root `Step` and its sub-steps. It contains checks that the current step can exit and the following step can enter based on input + output parameters and `Session.state_data`.

### `stepflow`
- This is the main dependency to include for libraries. 
- Its only purpose is to re-export StepFlow components for ease of use.
- A `prelude` is defined for commonly used traits

## Internal Dependency Order

| Crate                | Dependency           |
| -------------------- | -------------------- |
| `stepflow-test-util` | none                 |
| `stepflow-base`      | `stepflow-test-util` |
| `stepflow-data`      | `stepflow-base`      |
| `stepflow-step`      | `stepflow-data`      |
| `stepflow-action`    | `stepflow-step`      |
| `stepflow-session`   | `stepflow-action`    |

