use std::{iter::FusedIterator, marker::PhantomData};

use crate::*;

/// A helper function to map the internal iterator's output values into correct ones
fn map_next<Ticket, Id, T>(input: (usize, (Id, T))) -> (Ticket, Id, T)
where
    Ticket: RegistryTicket,
{
    let (idx, (id, val)) = input;
    (Ticket::from_index(idx).unwrap(), id, val)
}

/// A referencing iterator over the values of a [`RegistryManager`]
///
/// The iterator item-type is `(Ticket, &'a Identifier, &'a T)`
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    pub(crate) iter: std::iter::Enumerate<indexmap::map::Iter<'a, Identifier, T>>,
    pub(crate) _phantom: PhantomData<*const Ticket>,
}

impl<'a, T, Ticket, Identifier> Iterator for Iter<'a, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    type Item = (Ticket, &'a Identifier, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(map_next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(map_next)
    }

    fn last(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(map_next)
    }

    fn collect<C>(self) -> C
    where
        C: FromIterator<Self::Item>,
    {
        self.iter.map(map_next).collect()
    }
}

impl<T, Ticket, Identifier> DoubleEndedIterator for Iter<'_, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(map_next)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(map_next)
    }
}

impl<T, Ticket, Identifier> ExactSizeIterator for Iter<'_, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, Ticket, Identifier> FusedIterator for Iter<'_, T, Ticket, Identifier> where
    Ticket: RegistryTicket
{
}

/// A mutable iterator over the values of a [`RegistryManager`]
///
/// The iterator item-type is `(Ticket, &'a Identifier, &'a mut T)`
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IterMut<'a, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    pub(crate) iter: std::iter::Enumerate<indexmap::map::IterMut<'a, Identifier, T>>,
    pub(crate) _phantom: PhantomData<*const Ticket>,
}

impl<'a, T, Ticket, Identifier> Iterator for IterMut<'a, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    type Item = (Ticket, &'a Identifier, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(map_next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(map_next)
    }

    fn last(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(map_next)
    }

    fn collect<C>(self) -> C
    where
        C: FromIterator<Self::Item>,
    {
        self.iter.map(map_next).collect()
    }
}

impl<T, Ticket, Identifier> DoubleEndedIterator for IterMut<'_, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(map_next)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(map_next)
    }
}

impl<T, Ticket, Identifier> ExactSizeIterator for IterMut<'_, T, Ticket, Identifier>
where
    Ticket: RegistryTicket,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, Ticket, Identifier> FusedIterator for IterMut<'_, T, Ticket, Identifier> where
    Ticket: RegistryTicket
{
}
