use exonum::{
    crypto::PublicKey,
    storage::Fork,
    proto::ProtobufConvert,
};
use serde_json::{self, Value};
use crate::block::schema::Schema;
use super::CommonError;
use crate::error::CError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Permission {
    All,

    UserCreate,
    UserUpdate,
    UserAdminUpdate,
    UserSetPubkey,
    UserDelete,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    SuperAdmin,
    IdentityAdmin,
    User,
}

impl Role {
    pub fn permissions(&self) -> Vec<Permission> {
        match *self {
            Role::SuperAdmin => {
                vec![Permission::All]
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
            Role::User => {
                vec![
                    Permission::UserUpdate,
                ]
            }
        }
    }

    pub fn has_permission(&self, perm: &Permission) -> bool {
        self.permissions().contains(perm)
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
            .map_err(|_| From::from(CError::InvalidRole))
    }
}


pub fn check(schema: &mut Schema<&mut Fork>, pubkey: &PublicKey, permission: Permission) -> Result<(), CommonError> {
    if let Some(user) = schema.get_user_by_pubkey(pubkey) {
        for role in &user.roles {
            if role.has_permission(&permission) || role.has_permission(&Permission::All) {
                return Ok(())
            }
        }
    }
    Err(CommonError::InsufficientPrivileges)
}

