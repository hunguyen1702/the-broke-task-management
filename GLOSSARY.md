# Repository Task Management

Repository-local work planning and coordination between the human user and coding agents.

## Language

### Work and lifecycle

**Task**:
A tracked work item of any supported type, including an epic, story, bug, or the task type itself. When the type matters, use “task of type Task.”
_Avoid_: Using task to mean only the Task type without qualification.

**Status**:
A task's position in the repository's configured workflow, with a name and a repository-wide completion meaning shared by every task using that status. Status is independent of claim ownership and archive state.
_Avoid_: Using status to mean ownership or archive state.

**Completed status**:
A workflow status designated as completed, including outcomes such as Done, Cancelled, or Won't fix. Completion in this sense does not necessarily mean the work was delivered.
_Avoid_: Using Done as a synonym for every completed status.

**Incomplete status**:
A workflow status that is not designated as completed. A task with an incomplete status may still be effectively completed through archive.
_Avoid_: Using incomplete status to mean available work.

**Effective completion**:
The condition that satisfies downstream dependencies: a task is archived or has a completed status. It is independent of whether the task still has a claim.
_Avoid_: Using completed without distinguishing status completion from effective completion when the distinction matters.

**Active task**:
A task that is not archived, regardless of its status, claim, or availability.
_Avoid_: Using active to mean in progress, incomplete, claimed, or available.

**Archived task**:
A retained task removed from active planning with an archive reason, counted as effectively completed, and carrying no active claim. Its workflow status is preserved.
_Avoid_: Deleted task, or treating archived as a workflow status.

**Archive reason**:
The explanation for a task's current archived state. Supplemental comments are separate context, and the reason is cleared when the task is unarchived.
_Avoid_: Using a comment as a synonym for the archive reason.

**Unarchive**:
The return of an archived task to active planning with its existing status and without restoring its former claim. Its effective completion then depends on its status.
_Avoid_: Using reopen when it implies changing status or restoring ownership.

### Scope and execution relationships

**Hierarchy**:
The optional, single-parent organization of tasks by scope or containment, with no cycles. Hierarchy carries no implied execution order, completion roll-up, or ownership.
_Avoid_: Dependency tree, execution order.

**Parent**:
A task's direct container in the hierarchy, subject to the allowed task-type relationships. A task has at most one parent and may have none.
_Avoid_: Using parent to mean prerequisite.

**Child**:
A task directly contained by a parent in the hierarchy. Containment alone gives it no inherited status, archive state, claim, or availability.
_Avoid_: Using child to mean downstream task.

**Descendant**:
A task contained below another task through one or more parent-child relationships. A direct child is a descendant; deeper descendants are indirect.
_Avoid_: Using descendant to mean a downstream dependency.

**Dependency**:
A directed, mandatory execution relationship in which a downstream task requires an upstream task to be effectively completed. Tasks may have multiple dependencies, independently of hierarchy, and dependency cycles are forbidden.
_Avoid_: Parent-child relationship, optional prerequisite.

**Upstream task**:
The prerequisite task in a dependency: if A depends on B, B is directly upstream of A. Indirect upstream tasks are prerequisites reached through additional dependency relationships.
_Avoid_: Using upstream to mean parent or ancestor in the hierarchy.

**Downstream task**:
The dependent task in a dependency: if A depends on B, A is directly downstream of B. Indirect downstream tasks are dependents reached through additional dependency relationships.
_Avoid_: Using downstream to mean child or descendant in the hierarchy.

### Coordination and readiness

**Claim**:
An agent's current work ownership of a task, independent of status and without automatic expiry. A claim is a coordination signal rather than an exclusive right to edit the task.
_Avoid_: Assignment, lease, edit lock.

**Claim owner**:
The agent holding a task's current claim.
_Avoid_: Using author or updater to mean claim owner.

**Unclaim**:
The explicit release of a task's claim, leaving its status unchanged. Successful archive also releases a claim as part of removing the task from active planning.
_Avoid_: Using completion to mean claim release.

**Available task**:
An active task with an incomplete status, no claim, and every direct upstream dependency effectively completed. Availability describes current eligibility for a new claim, rather than a reservation or ownership guarantee.
_Avoid_: Using unclaimed or incomplete alone to mean available.

**Unavailable task**:
A task that fails at least one availability condition because it is archived, has a completed status, is claimed, or has an unresolved direct dependency. Several reasons may apply at once.
_Avoid_: Using dependency-blocked to mean every unavailable task.

**Satisfied dependency**:
A dependency whose direct upstream task is effectively completed, regardless of that upstream task's claim or its own unresolved dependencies. Having every dependency satisfied does not by itself make a task available.
_Avoid_: Using satisfied dependencies to mean available work.

**Unresolved dependency**:
A dependency whose upstream task is not effectively completed. It prevents the downstream task from being available without cancelling an existing downstream claim.
_Avoid_: Using blocked to mean only an unresolved dependency when other availability conditions may be involved.

**Dependency-blocked task**:
A task with at least one unresolved direct upstream dependency. This condition can coexist with a claim, a completed status, or archive.
_Avoid_: Using blocked without qualification to conflate dependency blocking with all reasons for unavailability.
