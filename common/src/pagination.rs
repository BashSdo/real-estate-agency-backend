//! Abstractions for pagination.

use std::fmt;

/// Generic pagination connection.
#[derive(Clone, Debug)]
pub struct Connection<C, I> {
    /// [`Edge`]s in this [`Connection`].
    pub edges: Vec<Edge<C, I>>,

    /// [`Kind`] of this [`Connection`].
    pub kind: Kind,

    /// Indicator whether this [`Connection`] has more nodes.
    pub has_more: bool,
}

/// A page in a [`Connection`].
pub type Page<C, I> = Connection<C, I>;

impl<C, I> Connection<C, I> {
    /// Creates a new [`Connection`] from the provided [`Edge`]s.
    #[must_use]
    pub fn new(
        args: &Arguments<C>,
        edges: impl IntoIterator<Item = impl Into<Edge<C, I>>>,
        has_more: bool,
    ) -> Self {
        Self {
            edges: edges.into_iter().map(Into::into).collect::<Vec<_>>(),
            kind: args.kind(),
            has_more,
        }
    }

    /// Returns [`PageInfo`] of this [`Connection`].
    #[must_use]
    pub fn page_info(&self) -> PageInfo<C>
    where
        C: Clone,
    {
        PageInfo {
            end_cursor: self.edges.last().map(|e| e.cursor.clone()),
            has_next_page: self.has_more && self.kind.is_forward(),
            has_previous_page: self.has_more && self.kind.is_backward(),
        }
    }
}

/// Information about a page in a [`Connection`].
#[derive(Clone, Copy, Debug)]
pub struct PageInfo<C> {
    /// Last cursor on this page.
    pub end_cursor: Option<C>,

    /// Indicator whether [`Connection`] has a next page.
    pub has_next_page: bool,

    /// Indicator whether [`Connection`] has a previous page.
    pub has_previous_page: bool,
}

/// An edge in a [`Connection`].
#[derive(Clone, Copy, Debug)]
pub struct Edge<C, I> {
    /// Cursor of this [`Edge`].
    pub cursor: C,

    /// Node of this [`Edge`].
    pub node: I,
}

impl<C, I> From<(C, I)> for Edge<C, I> {
    fn from((cursor, node): (C, I)) -> Self {
        Self { cursor, node }
    }
}

/// Pagination arguments.
#[derive(Clone, Copy, Debug)]
pub enum Arguments<C> {
    /// Forward pagination.
    Forward {
        /// Number of items to return.
        first: usize,

        /// Cursor after which to return items.
        after: Option<C>,

        /// Indicator whether the `after` cursor should be included in the
        /// result.
        including: bool,
    },

    /// Backward pagination.
    Backward {
        /// Number of items to return.
        last: usize,

        /// Cursor before which to return items.
        before: Option<C>,

        /// Indicator whether the `before` cursor should be included in the
        /// result.
        including: bool,
    },
}

impl<C> Arguments<C> {
    /// Creates a new [`Arguments`].
    pub fn new<Num>(
        first: Option<Num>,
        after: Option<C>,
        last: Option<Num>,
        before: Option<C>,
        default: Num,
    ) -> Option<Self>
    where
        C: PartialEq + fmt::Debug,
        Num: TryInto<usize> + fmt::Debug,
    {
        Some(match (first, after, last, before) {
            (None, None, None, None) => Self::Forward {
                first: default.try_into().ok()?,
                after: None,
                including: false,
            },
            (Some(first), None, None, None) => Self::Forward {
                first: first.try_into().ok()?,
                after: None,
                including: false,
            },
            (Some(first), Some(after), None, None) => Self::Forward {
                first: first.try_into().ok()?,
                after: Some(after),
                including: false,
            },
            (Some(first), Some(after), None, Some(before))
                if after == before =>
            {
                Self::Forward {
                    first: first.try_into().ok()?,
                    after: Some(after),
                    including: true,
                }
            }
            (None, None, Some(last), None) => Self::Backward {
                last: last.try_into().ok()?,
                before: None,
                including: false,
            },
            (None, None, Some(last), Some(before)) => Self::Backward {
                last: last.try_into().ok()?,
                before: Some(before),
                including: false,
            },
            (None, Some(after), Some(last), Some(before))
                if after == before =>
            {
                Self::Backward {
                    last: last.try_into().ok()?,
                    before: Some(before),
                    including: true,
                }
            }
            (None, Some(after), None, Some(before)) if after == before => {
                Self::Forward {
                    first: 1,
                    after: Some(after),
                    including: true,
                }
            }
            _ => return None,
        })
    }

    /// Returns exact cursor requested by this [`Arguments`].
    pub fn exact_cursor(&self) -> Option<&C> {
        match self {
            Self::Forward {
                first: 1,
                after,
                including: true,
            } => after.as_ref(),
            Self::Backward {
                last: 1,
                before,
                including: true,
            } => before.as_ref(),
            Self::Forward { .. } | Self::Backward { .. } => None,
        }
    }

    /// Returns cursor requested by this [`Arguments`].
    #[must_use]
    pub fn cursor(&self) -> Option<&C> {
        match self {
            Self::Forward { after, .. } => after.as_ref(),
            Self::Backward { before, .. } => before.as_ref(),
        }
    }

    /// Returns [`Kind`] of pagination this [`Arguments`] requests.
    pub fn kind(&self) -> Kind {
        match *self {
            Self::Forward { including, .. } => {
                if including {
                    Kind::ForwardIncluding
                } else {
                    Kind::Forward
                }
            }
            Self::Backward { including, .. } => {
                if including {
                    Kind::BackwardIncluding
                } else {
                    Kind::Backward
                }
            }
        }
    }

    /// Returns limit requested by this [`Arguments`].
    #[must_use]
    pub fn limit(&self) -> usize {
        match *self {
            Self::Forward { first, .. } => first,
            Self::Backward { last, .. } => last,
        }
    }
}

/// Pagination selector.
#[derive(Clone, Copy, Debug)]
pub struct Selector<C, F> {
    /// Pagination [`Arguments`].
    pub arguments: Arguments<C>,

    /// Additional filter being applied to the result.
    pub filter: F,
}

/// Kind of pagination.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// Forward pagination.
    Forward,

    /// Forward pagination including the cursor.
    ForwardIncluding,

    /// Backward pagination.
    Backward,

    /// Backward pagination including the cursor.
    BackwardIncluding,
}

impl Kind {
    /// Returns whether this [`Kind`] is forward.
    #[must_use]
    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward | Self::ForwardIncluding)
    }

    /// Returns whether this [`Kind`] is backward.
    #[must_use]
    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward | Self::BackwardIncluding)
    }

    /// Returns comparison operator representing this [`Kind`].
    #[must_use]
    pub const fn operator(&self) -> &'static str {
        match self {
            Self::Forward => ">",
            Self::ForwardIncluding => ">=",
            Self::Backward => "<",
            Self::BackwardIncluding => "<=",
        }
    }

    /// Returns order representing this [`Kind`].
    #[must_use]
    pub const fn order(&self) -> Order {
        match self {
            Self::Forward | Self::ForwardIncluding => Order::Ascending,
            Self::Backward | Self::BackwardIncluding => Order::Descending,
        }
    }
}

/// Order of pagination.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Order {
    /// Ascending order.
    Ascending,

    /// Descending order.
    Descending,
}

impl Order {
    #[cfg(feature = "postgres")]
    /// Returns SQL operator representing this [`Order`].
    #[must_use]
    pub const fn sql(&self) -> &'static str {
        match self {
            Self::Ascending => "ASC",
            Self::Descending => "DESC",
        }
    }
}

/// Defines pagination types.
#[expect(clippy::module_name_repetitions, reason = "more readable")]
#[macro_export]
macro_rules! define_pagination {
    ($cursor:ty, $node:ty, $filter:ty) => {
        #[doc = "Edge of a [`Connection`]."]
        pub type Edge = $crate::pagination::Edge<$cursor, $node>;

        #[doc = "A [`Connection`] of [`$node`]s."]
        pub type Connection = $crate::pagination::Connection<$cursor, $node>;

        #[doc = "A [`Page`] of [`$node`]s."]
        pub type Page = $crate::pagination::Page<$cursor, $node>;

        #[doc = "An information about a [`Page`]."]
        pub type PageInfo = $crate::pagination::PageInfo<$cursor>;

        #[doc = "Arguments for selecting a [`Page`]."]
        pub type Arguments = $crate::pagination::Arguments<$cursor>;

        #[doc = "[`Page`] selector."]
        pub type Selector = $crate::pagination::Selector<$cursor, $filter>;
    };
}
