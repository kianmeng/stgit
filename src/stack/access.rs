// SPDX-License-Identifier: GPL-2.0-only

//! The [`StackStateAccess`] trait allows uniform access to stack information for
//! stack-like objects.

use super::{
    iter::{AllPatches, BothPatches},
    state::PatchState,
};
use crate::patchname::PatchName;

/// Trait for accessing stack state.
///
/// Both `Stack` and `StackTransaction` implement this interface.
pub(crate) trait StackStateAccess<'repo> {
    /// Get slice of applied patch names.
    fn applied(&self) -> &[PatchName];

    /// Get slice of unapplied patch names.
    fn unapplied(&self) -> &[PatchName];

    /// Get slice of hidden patch names.
    fn hidden(&self) -> &[PatchName];

    /// Get patch state for given patch name.
    fn get_patch(&self, patchname: &PatchName) -> &PatchState<'repo>;

    /// Test whether given patch name exists in the stack.
    ///
    /// N.B. use [`StackStateAccess::collides()`] to test for potential patch name
    /// collisions.
    fn has_patch(&self, patchname: &PatchName) -> bool;

    /// Test whether given patch name collides with an existing patch name.
    ///
    /// A patch name collides if it is different from only by case from a patch in the
    /// stack.
    fn collides(&self, patchname: &PatchName) -> Option<&PatchName> {
        self.all_patches().find(|pn| patchname.collides(pn))
    }

    /// Get stack's top commit, or base if no applied patches.
    fn top(&self) -> &git2::Commit<'repo>;

    /// Get recorded head of the stack.
    ///
    /// N.B. this is probably not what you want. See also [`crate::stack::Stack::branch_head`].
    fn head(&self) -> &git2::Commit<'repo>;

    /// Get stack's base commit.
    fn base(&self) -> &git2::Commit<'repo>;

    /// Get the commit for the given patch name.
    fn get_patch_commit(&self, patchname: &PatchName) -> &git2::Commit<'repo> {
        &self.get_patch(patchname).commit
    }

    /// Test whether given patch name is applied.
    fn is_applied(&self, patchname: &PatchName) -> bool {
        self.applied().contains(patchname)
    }

    /// Test whether given patch name is unapplied.
    fn is_unapplied(&self, patchname: &PatchName) -> bool {
        self.unapplied().contains(patchname)
    }

    /// Test whether given patch name is hidden.
    fn is_hidden(&self, patchname: &PatchName) -> bool {
        self.hidden().contains(patchname)
    }

    /// Iterator over all patch names: applied, unapplied, and hidden.
    fn all_patches(&self) -> AllPatches<'_> {
        AllPatches::new(self.applied(), self.unapplied(), self.hidden())
    }

    /// Iterator over applied and unapplied patch names.
    fn applied_and_unapplied(&self) -> BothPatches<'_> {
        BothPatches::new(self.applied(), self.unapplied())
    }

    /// Iterator over unapplied and hidden patch names.
    fn unapplied_and_hidden(&self) -> BothPatches<'_> {
        BothPatches::new(self.unapplied(), self.hidden())
    }
}
