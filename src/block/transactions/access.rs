//! Defines a permission system for the entire blockchain. Essentially, allows
//! (or denies) top-level users to perform various actions depending on their
//! role in the system.

use exonum::{
    crypto::PublicKey,
    storage::Fork,
};
use crate::block::schema::Schema;
use crate::block::models::access::Permission;
use super::CommonError;

pub fn check(schema: &mut Schema<&mut Fork>, pubkey: &PublicKey, permission: Permission) -> Result<(), CommonError> {
    if let Some(user) = schema.get_user_by_pubkey(pubkey) {
        for role in &user.roles {
            if role.can(&permission) {
                return Ok(())
            }
        }
    }
    Err(CommonError::InsufficientPrivileges)
}

