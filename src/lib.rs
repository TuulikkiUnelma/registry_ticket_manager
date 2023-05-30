//! A manager of an [indexmap](https://crates.io/crates/indexmap) backed collection,
//! with both identifier based, and integer index ticket based value retrieval.
//!
//! The difference between this and the regular index map is the ticket based indexing which add
//! an extra layer of safety by making it more difficult to mix the indices of different collections.
//!
//! Items in the collection can't be removed, which means that old tickets will never be invalidated.
//!
//! # Examples
//!
//! ```
//! # use registry_ticket_manager_proc_macro::RegistryTicket;
//! use registry_ticket_manager::*;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Animal {
//!     pub category: String,
//!     pub sound: String,
//! }
//!
//! impl Animal {
//!     pub fn new(category: &str, sound: &str) -> Self {
//!         Self {
//!             category: category.to_string(),
//!             sound: sound.to_string(),
//!         }
//!     }
//! }
//!
//! #[derive(Debug, Clone, Copy, RegistryTicket)]
//! struct AnimalTicket(u16);
//!
//! type AnimalRegistry = RegistryManager<Animal, AnimalTicket>;
//!
//! fn main() {
//!     let mut man = AnimalRegistry::new();
//!
//!     let (cat, _old) = man.insert("cat", Animal::new("feline", "meow")).unwrap();
//!     let (dog, _old) = man.insert("dog", Animal::new("canine", "woof")).unwrap();
//!     let (cow, _old) = man.insert("cow", Animal::new("bovine", "moo")).unwrap();
//!
//!     let description = |ticket: AnimalTicket| {
//!         let (id, Animal { category, sound }) = man.get_ticket_full(ticket);
//!         format!("A {id} is a {category} and it goes {sound}!")
//!     };
//!
//!     assert_eq!(description(cat), "A cat is a feline and it goes meow!");
//!     assert_eq!(description(dog), "A dog is a canine and it goes woof!");
//!     assert_eq!(description(cow), "A cow is a bovine and it goes moo!");
//! }
//! ```

use indexmap::{map::Entry, IndexMap};
use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

/// A manager of arbitrary values with both identifier keys and index based tickets
///
/// It is implemented with an [`IndexMap`] from the crate [indexmap](https://crates.io/crates/indexmap).
///
/// A ticket type should be used for only *one* registry manager value, otherwise using those
/// ticket values might end up getting mixed with each other.
///
/// # Examples
///
/// ```
/// # use registry_ticket_manager_proc_macro::RegistryTicket;
/// use registry_ticket_manager::*;
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct Animal {
///     pub category: String,
///     pub sound: String,
/// }
///
/// impl Animal {
///     pub fn new(category: &str, sound: &str) -> Self {
///         Self {
///             category: category.to_string(),
///             sound: sound.to_string(),
///         }
///     }
/// }
///
/// #[derive(Debug, Clone, Copy, RegistryTicket)]
/// struct AnimalTicket(u16);
///
/// type AnimalRegistry = RegistryManager<Animal, AnimalTicket>;
///
/// fn main() {
///     let mut man = RegistryManager::new();
///
///     let (cat, _old) = man.insert("cat", Animal::new("feline", "meow")).unwrap();
///     let (dog, _old) = man.insert("dog", Animal::new("canine", "woof")).unwrap();
///     let (cow, _old) = man.insert("cow", Animal::new("bovine", "moo")).unwrap();
///
///     let description = |ticket: AnimalTicket| {
///         let (id, Animal { category, sound }) = man.get_ticket_full(ticket);
///         format!("A {id} is a {category} and it goes {sound}!")
///     };
///
///     assert_eq!(description(cat), "A cat is a feline and it goes meow!");
///     assert_eq!(description(dog), "A dog is a canine and it goes woof!");
///     assert_eq!(description(cow), "A cow is a bovine and it goes moo!");
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryManager<T, Ticket, Identifier = String>
where
    Ticket: RegistryTicket,
    Identifier: Hash + Eq,
{
    map: IndexMap<Identifier, T>,
    _phantom: PhantomData<Ticket>,
}

/// A registry manager ticket
///
/// This trait should be the only way to construct a ticket value,
/// and they should never be constructed outside of the [`RegistryManager`]'s methods.
/// In addition, the internal index value should be immutable and never changed.
///
/// Tickets of one type should only ever be used with one registry manager value, and never mixed.
///
/// Breaking any of these preconditions might create invalid tickets,
/// which are likely to cause undefined behaviour and out-of-bound reads and writes when used.
///
/// This trait can be auto-derived for newtype structs whose value is an unsigned integer:
///
/// ```
/// # use registry_ticket_manager_proc_macro::RegistryTicket;
/// # use registry_ticket_manager::RegistryTicket;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, RegistryTicket)]
/// struct Ticket(u16);
/// ```
///
/// The type of the wrapped value isn't checked by the auto-derive, but you should always use
/// a rust builtin unsigned integer type, either smaller or of equal size to `usize`.
pub trait RegistryTicket: Sized {
    /// Converts the given index number into a ticket
    ///
    /// This constructor should only be called inside the [`RegistryManager`]'s methods, never by the user,
    /// and the values of this type should never be constructed by any other means.
    ///
    /// This constructor can fail if the given index can't be converted,
    /// like for example if it is too big and outside its value range.
    ///
    /// This constructor must be deterministic for as long as it's used,
    /// ie. a specific input value always gives the same output.
    fn from_index(index: usize) -> Option<Self>;

    /// Converts this ticket into the given index number
    ///
    /// Must return the same value as it was constructed from in [`from_index`].
    fn to_index(&self) -> usize;
}

impl<T, Ticket, Identifier> RegistryManager<T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
    Identifier: Hash + Eq,
{
    /// Creates a new empty registry manager
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of stored values
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the registry manager is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns whether it is still possible to insert new values
    ///
    /// Equivalent to `[ticket type]::from_index(self.len()).is_some()`.
    pub fn can_insert(&self) -> bool {
        Ticket::from_index(self.len()).is_some()
    }

    /// Return if the given identifier (or something equal to it) exists in the registry
    pub fn contains_id(&self, id: &Identifier) -> bool {
        self.map.contains_key(id)
    }

    /// Inserts the value to the registry with the given identifier
    ///
    /// Returns the ticket, and if the given identifier already had a value, returns that as well.
    ///
    /// Returns `None` if the index of the would be inserted value could not be converted
    /// into a ticket by [`RegistryTicket::from_index`], without modifying the internal map.
    pub fn insert(&mut self, id: Identifier, value: T) -> Option<(Ticket, Option<T>)> {
        let entry = self.map.entry(id);
        let ticket = Ticket::from_index(entry.index())?;

        Some(match entry {
            Entry::Occupied(mut e) => (ticket, Some(e.insert(value))),
            Entry::Vacant(e) => {
                e.insert(value);
                (ticket, None)
            }
        })
    }

    /// Returns a reference to the value associated with the given id
    pub fn get_id(&self, id: &Identifier) -> Option<&T> {
        self.map.get(id)
    }

    /// Returns the ticket, a reference to the identifier,
    /// and a reference the value associated with the given identifier
    pub fn get_id_full(&self, id: &Identifier) -> Option<(Ticket, &Identifier, &T)> {
        let (idx, id, val) = self.map.get_full(id)?;
        Some((Ticket::from_index(idx)?, id, val))
    }

    /// Returns a mutable reference to the value associated with the given id
    pub fn get_id_mut(&mut self, id: &Identifier) -> Option<&mut T> {
        self.map.get_mut(id)
    }

    /// Returns the ticket, a reference to the identifier,
    /// and a mutable reference the value associated with the given identifier
    pub fn get_id_full_mut(&mut self, id: &Identifier) -> Option<(Ticket, &Identifier, &mut T)> {
        let (idx, id, val) = self.map.get_full_mut(id)?;
        Some((Ticket::from_index(idx)?, id, val))
    }

    /// Returns a reference to the value associated with the given ticket
    ///
    /// Assumes that the given ticket is valid.
    pub fn get_ticket(&self, ticket: Ticket) -> &T {
        &self.map[ticket.to_index()]
    }

    /// Returns references to the identifier and the value associated with the given ticket
    ///
    /// Assumes that the given ticket is valid.
    pub fn get_ticket_full(&self, ticket: Ticket) -> (&Identifier, &T) {
        self.map.get_index(ticket.to_index()).unwrap()
    }

    /// Returns a mutable reference to the value associated with the given ticket
    ///
    /// Assumes that the given ticket is valid.
    pub fn get_ticket_mut(&mut self, ticket: Ticket) -> &mut T {
        &mut self.map[ticket.to_index()]
    }

    /// Returns a reference to the identifier and a mutable reference to the value
    /// associated with the given ticket
    ///
    /// Assumes that the given ticket is valid.
    pub fn get_ticket_full_mut(&mut self, ticket: Ticket) -> (&Identifier, &mut T) {
        let (id, val) = self.map.get_index_mut(ticket.to_index()).unwrap();
        (id, val)
    }

    /// Returns the ticket of the given identifier, if it exists
    pub fn get_ticket_of(&self, id: &Identifier) -> Option<Ticket> {
        self.map.get_index_of(id).and_then(Ticket::from_index)
    }
}

impl<T, Ticket, Identifier> Default for RegistryManager<T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
    Identifier: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Ticket, Identifier> Index<Ticket> for RegistryManager<T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
    Identifier: Hash + Eq,
{
    type Output = T;
    /// Returns a reference to the value associated by the ticket
    ///
    /// Assumes that the given ticket is valid.
    fn index(&self, ticket: Ticket) -> &Self::Output {
        self.get_ticket(ticket)
    }
}

impl<T, Ticket, Identifier> IndexMut<Ticket> for RegistryManager<T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
    Identifier: Hash + Eq,
{
    /// Returns a mutable reference to the value associated by the ticket
    ///
    /// Assumes that the given ticket is valid.
    fn index_mut(&mut self, ticket: Ticket) -> &mut Self::Output {
        self.get_ticket_mut(ticket)
    }
}
