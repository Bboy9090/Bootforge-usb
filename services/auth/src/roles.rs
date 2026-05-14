use std::collections::HashSet;

// Role management implementation
pub struct RoleManager {
    roles: HashSet<String>,
}

impl RoleManager {
    pub fn new() -> Self {
        RoleManager {
            roles: HashSet::new(),
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    pub fn assign_role(&mut self, role: String) {
        self.roles.insert(role);
    }
}
