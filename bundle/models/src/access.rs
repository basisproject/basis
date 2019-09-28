use exonum::proto::ProtobufConvert;
use serde_json::{self, Value};
use error::BError;

/// Define the system-wide permissions.
///
/// Note there may be per-model permissions that are handled separately.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Permission {
    All,
    AllBut(Vec<Permission>),

    TimeTravel,

    UserCreate,
    UserUpdate,
    UserAdminUpdate,
    UserSetPubkey,
    UserDelete,

    CompanyCreateSyndicate,
    CompanyCreatePrivate,
    CompanySetApproved,
    CompanyAdminUpdate,
    CompanyAdminDelete,
    CompanySetType,
    CompanyUpdateMembers,
    CompanyClockIn,
    CompanyClockOut,
    CompanyAdminClock,

    ProductCreate,
    ProductUpdate,
    ProductDelete,
    ProductAdminUpdate,
    ProductAdminDelete,

    OrderCreate,
    OrderUpdate,
    OrderAdminUpdate,
}

/// Define the system-wide roles users can have.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    SuperAdmin,
    TimeTraveller,
    IdentityAdmin,
    CompanyAdmin,
    ProductAdmin,
    OrderAdmin,
    Bank,
    User,
}

impl Role {
    pub fn permissions(&self) -> Vec<Permission> {
        match *self {
            Role::SuperAdmin => {
                vec![
                    Permission::AllBut(vec![Permission::TimeTravel]),
                ]
            },
            Role::TimeTraveller => {
                vec![Permission::TimeTravel]
            },
            Role::IdentityAdmin => {
                vec![
                    Permission::UserCreate,
                    Permission::UserUpdate,
                    Permission::UserAdminUpdate,
                    Permission::UserSetPubkey,
                    Permission::UserDelete,
                ]
            },
            Role::CompanyAdmin => {
                vec![
                    Permission::CompanySetApproved,
                    Permission::CompanyAdminUpdate,
                    Permission::CompanyAdminDelete,
                ]
            }
            Role::ProductAdmin => {
                vec![
                    Permission::ProductAdminUpdate,
                    Permission::ProductAdminDelete,
                ]
            }
            Role::OrderAdmin => {
                vec![
                    Permission::OrderAdminUpdate,
                ]
            }
            Role::Bank => {
                vec![
                    Permission::CompanySetType,
                    Permission::CompanySetApproved,
                ]
            },
            Role::User => {
                vec![
                    Permission::UserUpdate,
                    Permission::UserDelete,
                    Permission::CompanyCreateSyndicate,
                    Permission::CompanyCreatePrivate,
                    Permission::CompanyUpdateMembers,
                    Permission::CompanyClockIn,
                    Permission::CompanyClockOut,
                    Permission::ProductCreate,
                    Permission::ProductUpdate,
                    Permission::ProductDelete,
                    Permission::OrderCreate,
                    Permission::OrderUpdate,
                ]
            }
        }
    }

    pub fn can(&self, perm: &Permission) -> bool {
        for p in &self.permissions() {
            match p {
                Permission::All => {
                    return true;
                }
                Permission::AllBut(x) => {
                    if x.contains(perm) {
                        return false;
                    }
                    return true;
                }
                _ => {
                    if p == perm {
                        return true
                    }
                }
            }
        }
        false
    }
}

impl ProtobufConvert for Role {
    type ProtoStruct = String;

    fn to_pb(&self) -> Self::ProtoStruct {
        match serde_json::to_value(self) {
            Ok(Value::String(x)) => x,
            _ => String::from("<invalid-role>"),
        }
    }

    fn from_pb(pb: Self::ProtoStruct) -> Result<Self, failure::Error> {
        serde_json::from_value::<Role>(Value::String(pb))
            .map_err(|_| From::from(BError::InvalidRole))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn permissions_work() {
        let super_admin = Role::SuperAdmin;
        assert!(super_admin.can(&Permission::All));
        assert!(!super_admin.can(&Permission::TimeTravel));
        assert!(super_admin.can(&Permission::UserCreate));
        assert!(super_admin.can(&Permission::UserUpdate));
        assert!(super_admin.can(&Permission::UserAdminUpdate));
        assert!(super_admin.can(&Permission::UserSetPubkey));
        assert!(super_admin.can(&Permission::UserDelete));
        assert!(super_admin.can(&Permission::CompanyCreatePrivate));
        assert!(super_admin.can(&Permission::CompanyAdminUpdate));
        assert!(super_admin.can(&Permission::CompanyAdminDelete));
        assert!(super_admin.can(&Permission::CompanySetType));

        let traveller = Role::TimeTraveller;
        assert!(traveller.can(&Permission::TimeTravel));
        assert!(!traveller.can(&Permission::UserCreate));
        assert!(!traveller.can(&Permission::UserUpdate));
        assert!(!traveller.can(&Permission::UserAdminUpdate));
        assert!(!traveller.can(&Permission::UserSetPubkey));
        assert!(!traveller.can(&Permission::UserDelete));
        assert!(!traveller.can(&Permission::CompanyCreatePrivate));
        assert!(!traveller.can(&Permission::CompanyAdminUpdate));
        assert!(!traveller.can(&Permission::CompanyAdminDelete));
        assert!(!traveller.can(&Permission::CompanySetType));

        let comp_admin = Role::CompanyAdmin;
        assert!(!comp_admin.can(&Permission::TimeTravel));
        assert!(!comp_admin.can(&Permission::UserCreate));
        assert!(!comp_admin.can(&Permission::UserUpdate));
        assert!(!comp_admin.can(&Permission::UserAdminUpdate));
        assert!(!comp_admin.can(&Permission::UserSetPubkey));
        assert!(!comp_admin.can(&Permission::UserDelete));
        assert!(!comp_admin.can(&Permission::CompanyCreatePrivate));
        assert!(comp_admin.can(&Permission::CompanyAdminUpdate));
        assert!(comp_admin.can(&Permission::CompanyAdminDelete));
        assert!(!comp_admin.can(&Permission::CompanySetType));

        // TODO: ProductAdmin
        // TODO: OrderAdmin
    }
}

