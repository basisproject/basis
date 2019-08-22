//! Defines a permission system for the entire blockchain. Essentially, allows
//! (or denies) top-level users to perform various actions depending on their
//! role in the system.

use exonum::{
    crypto::PublicKey,
};
use exonum_merkledb::IndexAccess;
use crate::block::schema::Schema;
use crate::block::models::access::Permission;
use super::CommonError;

pub fn check<T>(schema: &mut Schema<T>, pubkey: &PublicKey, permission: Permission) -> Result<(), CommonError>
    where T: IndexAccess
{
    if let Some(user) = schema.get_user_by_pubkey(pubkey) {
        for role in &user.roles {
            if role.can(&permission) {
                return Ok(())
            }
        }
    }
    Err(CommonError::InsufficientPrivileges)
}

