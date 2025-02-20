use std::borrow::Cow;

use egui::Image;
use indexmap::IndexMap;

/// Where the decoration will be displayed. Used to provide context when requesting a decoration from an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationKind<'a> {
    List,
    DropDown,
    Other(&'a str),
}

/// An item that could be rendered in a view. Typically part of a collection of similar items stored in a model.
pub trait ViewItem: Clone {
    type DecorationDependencies;

    /// Indicates whether ANY item of this type could be decorated.
    /// If not, widgets may skip allocating space for decorations and use a simpler layout.
    #[allow(unused)]
    fn decorated(kind: DecorationKind<'_>) -> bool {
        false
    }

    /// Retrieve the display text for this item using the given dependencies.
    fn with_text<F, R>(&self, consumer: F) -> R
    where
        F: FnOnce(&str) -> R;

    /// Retrieve the decoration for this item and the recommended scale to display it with.
    /// The [DecorationKind] may be used to provide different decorations based on the context.
    #[allow(unused)]
    fn with_decoration<F, R>(
        &self,
        dependencies: &Self::DecorationDependencies,
        kind: DecorationKind<'_>,
        consumer: F,
    ) -> R
    where
        F: FnOnce(Option<Image>) -> R,
    {
        consumer(None)
    }
}

/// A [ViewItem] that has a unique ID distinguishing it from other items.
pub trait KeyedViewItem: ViewItem {
    /// Retrieve the key from this item.
    fn key(&self) -> Cow<'_, str>;

    fn set_key(&mut self, key: String);
}

/// An array-like of [ViewItem] that can be rendered in collection widgets.
pub trait ListModel<I> {
    /// Whether the list is empty.
    fn is_empty(&self) -> bool;

    /// The number of items in this model.
    fn len(&self) -> usize;

    /// Retrieve the item at the given index (if in bounds)
    fn item(&self, index: usize) -> Option<&I>;

    /// Retrieve a mutable reference to an item if the index is in bounds.
    fn item_mut(&mut self, index: usize) -> Option<&mut I>;

    /// Add an item to the end of this model (if supported)
    fn add(&mut self, item: I);

    /// Insert an item at the specified index (if in bounds)
    fn insert(&mut self, index: usize, item: I);

    /// Remove the item at the given index (if in bounds)
    fn remove(&mut self, index: usize);

    /// Swap items at the given indices (if in bounds)
    fn swap_items(&mut self, a: usize, b: usize);

    /// Copy the contents of index `a` to index `b`.
    fn copy(&mut self, a: usize, b: usize);

    /// Convert a row number to its index in the underlying collection.
    fn row_to_index(&self, row_number: usize) -> Option<usize>;
}

impl<I> ListModel<I> for Vec<I>
where
    I: ViewItem,
{
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn item(&self, index: usize) -> Option<&I> {
        self.get(index)
    }

    fn item_mut(&mut self, index: usize) -> Option<&mut I> {
        self.get_mut(index)
    }

    fn add(&mut self, item: I) {
        self.push(item);
    }

    fn insert(&mut self, index: usize, item: I) {
        if index <= self.len() {
            self.insert(index, item);
        }
    }

    fn remove(&mut self, index: usize) {
        if index < self.len() {
            self.remove(index);
        }
    }

    fn swap_items(&mut self, a: usize, b: usize) {
        if a < self.len() && b < self.len() {
            self.swap(a, b);
        }
    }

    fn copy(&mut self, a: usize, b: usize) {
        if a < self.len() && b < self.len() {
            self[b] = self[a].clone();
        }
    }

    fn row_to_index(&self, row_number: usize) -> Option<usize> {
        (0..self.len()).contains(&row_number).then_some(row_number)
    }
}

impl<I> ListModel<I> for IndexMap<String, I>
where
    I: KeyedViewItem,
{
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn item(&self, index: usize) -> Option<&I> {
        self.get_index(index).map(|(_, v)| v)
    }

    fn item_mut(&mut self, index: usize) -> Option<&mut I> {
        self.get_index_mut(index).map(|(_, v)| v)
    }

    fn add(&mut self, item: I) {
        let key = item.key();
        if !self.contains_key(key.as_ref()) {
            self.insert(key.into_owned(), item);
        }
    }

    fn insert(&mut self, index: usize, item: I) {
        if index <= self.len() {
            self.add(item);
            self.move_index(self.len() - 1, index);
        }
    }

    fn remove(&mut self, index: usize) {
        if index < self.len() {
            self.shift_remove_index(index);
        }
    }

    fn swap_items(&mut self, a: usize, b: usize) {
        if a < self.len() && b < self.len() {
            self.swap_indices(a, b);
        }
    }

    fn copy(&mut self, a: usize, b: usize) {
        if let Some(key) = self.get_index(b).map(|(k, _)| k).cloned() {
            if let Some(mut a) = self.get_index(a).map(|(_, v)| v).cloned() {
                a.set_key(key.clone());
                self.insert(key, a);
            }
        }
    }

    fn row_to_index(&self, row_number: usize) -> Option<usize> {
        (0..self.len()).contains(&row_number).then_some(row_number)
    }
}

/// A [ListModel] of items which have a unique ID.
pub trait KeyedListModel<I>: ListModel<I> {
    /// Retrieve the kind of a [ViewItem] from its key.
    fn index_of(&self, key: &str) -> Option<usize>;

    fn item_keyed(&self, key: &str) -> Option<&I> {
        self.index_of(key).and_then(|index| self.item(index))
    }

    fn contains(&self, key: &str) -> bool {
        self.index_of(key).is_some()
    }
}

impl<I> KeyedListModel<I> for IndexMap<String, I>
where
    I: KeyedViewItem,
{
    fn index_of(&self, key: &str) -> Option<usize> {
        self.get_index_of(key)
    }
}
