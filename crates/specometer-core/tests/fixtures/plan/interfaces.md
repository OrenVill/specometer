## Global Constraints

- none

### Task 1: Types

**Interfaces:**
- Produces: `pub struct UserId(pub u64);`

### Task 2: API

**Interfaces:**
- Consumes: `UserId`

- Produces: `pub fn get_user(id: UserId) -> User;`
