//! Marker types.

/// Marker type describing an entity creation.
#[derive(Clone, Copy, Debug)]
pub struct Creation;

/// Marker type describing an entity deletion.
#[derive(Clone, Copy, Debug)]
pub struct Deletion;
