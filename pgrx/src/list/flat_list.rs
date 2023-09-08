use crate::pg_sys;
use core::cmp;
use core::ffi;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Bound, Deref, DerefMut, RangeBounds};
use core::ptr::{self, NonNull};
use core::slice;

/// The List type from Postgres, lifted into Rust
/// Note: you may want the ListHead type
#[derive(Debug)]
pub enum List<T> {
    Nil,
    Cons(ListHead<T>),
}

/// A strongly-typed ListCell
#[repr(transparent)]
pub struct ListCell<T> {
    // It is important that we are able to treat this union as effectively synonymous with T!
    // Thus it is important that we
    // - do not hand out the ability to construct arbitrary ListCell<T>
    // - do not offer casting between types of List<T> (which offer [ListCell<T>])
    // - do not even upgrade from pg_sys::{List, ListCell} to pgrx::list::{List, ListCell}
    // UNLESS the relevant safety invariants are appropriately handled!
    // It is not even okay to do this for FFI! We must check any *mut pg_sys::List from FFI,
    // to guarantee it has the expected type tag, otherwise the union cells may be garbage.
    cell: pg_sys::ListCell,
    _type: PhantomData<T>,
}

// Note: the size of `ListCell<T>`'s generic `T` doesn't matter,
// thus it isn't acceptable to implement Enlist for a `T` larger than `pg_sys::ListCell`.
const _: () = {
    assert!(mem::size_of::<ListCell<u128>>() == mem::size_of::<pg_sys::ListCell>());
};

impl<T: Enlist> Deref for ListCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: A brief upgrade of readonly &ListCell<T> to writable *mut pg_sys::ListCell
        // may seem sus, but is fine: Enlist::apoptosis is defined as pure casting/arithmetic.
        // So the pointer begins and ends without write permission, and
        // we essentially just reborrow a ListCell as its inner field type
        unsafe { &*T::apoptosis(&self.cell as *const _ as *mut _) }
    }
}

impl<T: Enlist> DerefMut for ListCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we essentially just reborrow a ListCell as its inner field type which
        // only relies on pgrx::list::{Enlist, List, ListCell} maintaining safety invariants
        unsafe { &mut *T::apoptosis(&mut self.cell) }
    }
}

#[derive(Debug)]
pub struct ListHead<T> {
    list: NonNull<pg_sys::List>,
    _type: PhantomData<[T]>,
}

mod seal {
    pub trait Sealed {}
}

/// The bound to describe a type which may be used in a Postgres List
/// It must know what an appropriate type tag is, and how to pointer-cast to itself
///
/// # Safety
/// `List<T>` relies in various ways on this being correctly implemented.
/// Incorrect implementation can lead to broken Lists, UB, or "database hilarity".
///
/// Only realistically valid to implement for union variants of pg_sys::ListCell.
/// It's not even correct to impl for `*mut T`, as `*mut T` may be a fat pointer!
pub unsafe trait Enlist: seal::Sealed + Sized {
    // The appropriate list tag for this type.
    const LIST_TAG: pg_sys::NodeTag;

    /// From a pointer to the pg_sys::ListCell union, obtain a pointer to Self
    /// I think this isn't actually unsafe, it just has an unsafe impl invariant?
    /// It must be implemented with ptr::addr_of! or similar, without reborrowing
    /// so that it may be used without regard to whether a pointer is write-capable
    unsafe fn apoptosis(cell: *mut pg_sys::ListCell) -> *mut Self;

    /// Set a value into a `pg_sys::ListCell`
    ///
    /// This is used instead of Enlist::apoptosis, as it guarantees initializing the union
    /// according to the rules of Rust. In practice, this is probably the same,
    /// but this way I don't have to wonder, as this is a safe function.
    fn endocytosis(cell: &mut pg_sys::ListCell, value: Self);
}

impl seal::Sealed for *mut ffi::c_void {}
unsafe impl Enlist for *mut ffi::c_void {
    const LIST_TAG: pg_sys::NodeTag = pg_sys::NodeTag::T_List;

    unsafe fn apoptosis(cell: *mut pg_sys::ListCell) -> *mut *mut ffi::c_void {
        unsafe { ptr::addr_of_mut!((*cell).ptr_value) }
    }

    fn endocytosis(cell: &mut pg_sys::ListCell, value: Self) {
        cell.ptr_value = value;
    }
}

impl seal::Sealed for ffi::c_int {}
unsafe impl Enlist for ffi::c_int {
    const LIST_TAG: pg_sys::NodeTag = pg_sys::NodeTag::T_IntList;

    unsafe fn apoptosis(cell: *mut pg_sys::ListCell) -> *mut ffi::c_int {
        unsafe { ptr::addr_of_mut!((*cell).int_value) }
    }

    fn endocytosis(cell: &mut pg_sys::ListCell, value: Self) {
        cell.int_value = value;
    }
}

impl seal::Sealed for pg_sys::Oid {}
unsafe impl Enlist for pg_sys::Oid {
    const LIST_TAG: pg_sys::NodeTag = pg_sys::NodeTag::T_OidList;

    unsafe fn apoptosis(cell: *mut pg_sys::ListCell) -> *mut pg_sys::Oid {
        unsafe { ptr::addr_of_mut!((*cell).oid_value) }
    }

    fn endocytosis(cell: &mut pg_sys::ListCell, value: Self) {
        cell.oid_value = value;
    }
}

#[cfg(feature = "pg16")]
impl seal::Sealed for pg_sys::TransactionId {}
#[cfg(feature = "pg16")]
unsafe impl Enlist for pg_sys::TransactionId {
    const LIST_TAG: pg_sys::NodeTag = pg_sys::NodeTag::T_XidList;

    unsafe fn apoptosis(cell: *mut pg_sys::ListCell) -> *mut pg_sys::TransactionId {
        unsafe { ptr::addr_of_mut!((*cell).xid_value) }
    }

    fn endocytosis(cell: &mut pg_sys::ListCell, value: Self) {
        cell.xid_value = value;
    }
}

/// Note the absence of `impl Default for ListHead`:
/// it must initialize at least 1 element to be created at all
impl<T> Default for List<T> {
    fn default() -> List<T> {
        List::Nil
    }
}

impl<T: Enlist> List<T> {
    /// Attempt to obtain a `List<T>` from a `*mut pg_sys::List`
    ///
    /// This may be somewhat confusing:
    /// A valid List of any type is the null pointer, as in the Lisp `(car, cdr)` representation.
    /// This remains true even after significant reworks of the List type in Postgres 13, which
    /// cause it to internally use a "flat array" representation.
    ///
    /// Thus, this returns `Some` even if the List is NULL, because it is `Some(List::Nil)`,
    /// and returns `None` only if the List is non-NULL but downcasting failed!
    ///
    /// # Safety
    /// This assumes the pointer is either NULL or the NodeTag is valid to read,
    /// so it is not okay to call this on pointers to deallocated or uninit data.
    ///
    /// If it returns as `Some` and the List is more than zero length, it also asserts
    /// that the entire List's `elements: *mut ListCell` is validly initialized as `T`
    /// in each ListCell and that the List is allocated from a Postgres memory context.
    ///
    /// **Note:** This memory context must last long enough for your purposes.
    /// YOU are responsible for bounding its lifetime correctly.
    pub unsafe fn downcast_from_nullable(ptr: *mut pg_sys::List) -> Option<List<T>> {
        match NonNull::new(ptr) {
            None => Some(List::Nil),
            Some(list) => ListHead::downcast_from_ptr(list).map(|head| List::Cons(head)),
        }
    }

    /// Borrow an item from the slice at the index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_cells().get(index).map(Deref::deref)
    }

    /// Mutably borrow an item from the slice at the index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_cells_mut().get_mut(index).map(DerefMut::deref_mut)
    }

    /// Push, and if allocation is needed, allocate in a given context
    /// "Unstable" because this will probably receive breaking changes every week for a few weeks.
    ///
    /// # Safety
    ///
    /// Use the right context, don't play around.
    pub unsafe fn unstable_push_in_context(
        &mut self,
        value: T,
        context: pg_sys::MemoryContext,
    ) -> &mut ListHead<T> {
        match self {
            List::Nil => {
                // No silly reasoning, simply allocate a cache line for a list.
                let list_size = 64;
                let list: *mut pg_sys::List = pg_sys::MemoryContextAlloc(context, list_size).cast();
                (*list).type_ = T::LIST_TAG;
                (*list).length = 1;
                (*list).max_length = ((list_size - mem::size_of::<pg_sys::List>())
                    / mem::size_of::<pg_sys::ListCell>()) as _;
                (*list).elements = ptr::addr_of_mut!((*list).initial_elements).cast();
                T::apoptosis((*list).elements).write(value);
                *self = Self::downcast_from_nullable(list).unwrap();
                match self {
                    List::Cons(head) => head,
                    _ => unreachable!(),
                }
            }
            List::Cons(head) => head.push(value),
        }
    }

    /// Attempt to push or Err if it would allocate
    ///
    /// This exists primarily to allow working with a list with maybe-zero capacity.
    pub fn try_push(&mut self, value: T) -> Result<&mut ListHead<T>, &mut Self> {
        match self {
            List::Nil => Err(self),
            list if list.capacity() - list.len() == 0 => Err(list),
            List::Cons(head) => Ok(head.push(value)),
        }
    }

    /// Try to reserve space for N more items
    pub fn try_reserve(&mut self, items: usize) -> Result<&mut ListHead<T>, &mut Self> {
        match self {
            List::Nil => Err(self),
            List::Cons(head) => Ok(head.reserve(items)),
        }
    }

    // Iterate over part of the List while removing elements from it
    //
    // Note that if this removes the last item, it deallocates the entire list.
    // This is to maintain the Postgres List invariant that a 0-len list is always Nil.
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        // SAFETY: The Drain invariants are somewhat easier to maintain for List than Vec,
        // however, they have the complication of the Postgres List invariants
        let len = self.len();
        let drain_start = match range.start_bound() {
            Bound::Unbounded | Bound::Included(0) => 0,
            Bound::Included(first) => *first,
            Bound::Excluded(point) => point + 1,
        };
        let tail_start = match range.end_bound() {
            Bound::Unbounded => cmp::min(ffi::c_int::MAX as _, len),
            Bound::Included(last) => last + 1,
            Bound::Excluded(tail) => *tail,
        };
        let Some(tail_len) = len.checked_sub(tail_start) else {
            panic!("index out of bounds of list!")
        };
        // Let's issue our asserts before mutating state:
        assert!(drain_start <= len);
        assert!(tail_start <= len);
        assert!(drain_start + tail_len == len);

        // Postgres assumes Lists fit into c_int, check before shrinking
        assert!(tail_start <= ffi::c_int::MAX as _);
        assert!(drain_start + tail_len <= ffi::c_int::MAX as _);

        // If draining all, rip it out of place to contain broken invariants from panics
        let raw = if drain_start == 0 {
            mem::take(self).into_nullable()
        } else {
            // Leave it in place, but we need a pointer:
            match self {
                List::Nil => ptr::null_mut(),
                List::Cons(head) => head.list.as_ptr().cast(),
            }
        };

        // Remember to check that our raw ptr is non-null
        if raw != ptr::null_mut() {
            // Shorten the list to prohibit interaction with List's state after drain_start.
            // Note this breaks List repr invariants in the `drain_start == 0` case, but
            // we only consider returning the list ptr to `&mut self` if Drop is completed
            unsafe { (*raw).length = drain_start as _ };
            let cells_ptr = unsafe { (*raw).elements };
            let iter = unsafe {
                RawCellIter {
                    ptr: cells_ptr.add(drain_start).cast(),
                    end: cells_ptr.add(tail_start).cast(),
                }
            };
            Drain { tail_len: tail_len as _, tail_start: tail_start as _, raw, origin: self, iter }
        } else {
            // If it's not, produce the only valid choice: a 0-len iterator pointing to null
            // One last doublecheck for old paranoia's sake:
            assert!(tail_len == 0 && tail_start == 0 && drain_start == 0);
            Drain { tail_len: 0, tail_start: 0, raw, origin: self, iter: Default::default() }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_cells().into_iter().map(Deref::deref)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_cells_mut().into_iter().map(DerefMut::deref_mut)
    }
}

impl<T> List<T> {
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(head) => head.len(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(head) => head.capacity(),
        }
    }

    pub fn into_nullable(self) -> *mut pg_sys::List {
        match self {
            List::Nil => ptr::null_mut(),
            List::Cons(head) => head.list.as_ptr(),
        }
    }

    /// Borrow the List's slice of cells
    ///
    /// Note that like with Vec, this slice may move after appending to the List!
    /// Due to lifetimes this isn't a problem until unsafe Rust becomes involved,
    /// but with Postgres extensions it often does.
    ///
    /// Note that if you use this on a 0-item list, you get an empty slice, of course.
    pub fn as_cells(&self) -> &[ListCell<T>] {
        unsafe {
            match self {
                // No elements? No problem! Return a 0-sized slice
                List::Nil => slice::from_raw_parts(self as *const _ as _, 0),
                List::Cons(inner) => slice::from_raw_parts(inner.as_cells_ptr(), inner.len()),
            }
        }
    }

    /// Mutably borrow the List's slice of cells
    ///
    /// Includes the same caveats as with `List::as_cells`, but with "less" problems:
    /// `&mut` means you should not have other pointers to the list anyways.
    ///
    /// Note that if you use this on a 0-item list, you get an empty slice, of course.
    pub fn as_cells_mut(&mut self) -> &mut [ListCell<T>] {
        // SAFETY: Note it is unsafe to read a union variant, but safe to set a union variant!
        // This allows access to `&mut pg_sys::ListCell` to mangle a List's type in safe code.
        // Also note that we can't yield &mut [T] because Postgres Lists aren't tight-packed.
        // These facts are why the entire List type's interface isn't much simpler.
        //
        // This function is safe as long as ListCell<T> offers no way to corrupt the list,
        // and as long as we correctly maintain the length of the List's type.
        unsafe {
            match self {
                // No elements? No problem! Return a 0-sized slice
                List::Nil => slice::from_raw_parts_mut(self as *mut _ as _, 0),
                List::Cons(inner) => {
                    slice::from_raw_parts_mut(inner.as_mut_cells_ptr(), inner.len())
                }
            }
        }
    }
}

impl<T> ListHead<T> {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { self.list.as_ref().length as usize }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { self.list.as_ref().max_length as usize }
    }

    /// Borrow the List's slice of cells
    ///
    /// Note that like with Vec, this slice may move after appending to the List!
    /// Due to lifetimes this isn't a problem until unsafe Rust becomes involved,
    /// but with Postgres extensions it often does.
    pub fn as_cells(&self) -> &[ListCell<T>] {
        unsafe { slice::from_raw_parts(self.as_cells_ptr(), self.len()) }
    }

    pub fn as_cells_ptr(&self) -> *const ListCell<T> {
        unsafe { (*self.list.as_ptr()).elements.cast() }
    }

    pub fn as_mut_cells_ptr(&mut self) -> *mut ListCell<T> {
        unsafe { (*self.list.as_ptr()).elements.cast() }
    }
}

impl<T: Enlist> ListHead<T> {
    /// From a non-nullable pointer that points to a valid List, produce a ListHead of the correct type
    ///
    /// # Safety
    /// This assumes the NodeTag is valid to read, so it is not okay to call this on
    /// pointers to deallocated or uninit data.
    ///
    /// If it returns as `Some`, it also asserts the entire List is, across its length,
    /// validly initialized as `T` in each ListCell.
    pub unsafe fn downcast_from_ptr(list: NonNull<pg_sys::List>) -> Option<ListHead<T>> {
        (T::LIST_TAG == (*list.as_ptr()).type_).then_some(ListHead { list, _type: PhantomData })
    }

    pub fn push(&mut self, value: T) -> &mut Self {
        let list = unsafe { self.list.as_mut() };
        let pg_sys::List { length, max_length, elements, .. } = list;
        if *max_length - *length > 0 {
            // SAFETY: Our list must have been constructed following the list invariants
            // in order to actually get here, and we have confirmed as in-range of the buffer.
            let cell = unsafe { &mut *elements.add(*length as _) };
            T::endocytosis(cell, value);
            *length += 1;
        } else {
            // Reserve in this branch.
            let new_cap = max_length.saturating_mul(2);
            self.reserve(new_cap as _);
        }

        // Return `self` for convenience of `List::try_push`
        self
    }

    pub fn reserve(&mut self, size: usize) -> &mut Self {
        let list = unsafe { self.list.as_mut() };
        if ((list.max_length - list.length) as usize) < size {
            unsafe { grow_list(list, size + list.length as usize) };
        };
        self
    }
}

unsafe fn grow_list(list: &mut pg_sys::List, target: usize) {
    let alloc_size = target * mem::size_of::<pg_sys::ListCell>();
    if list.elements == ptr::addr_of_mut!(list.initial_elements).cast() {
        // first realloc, we can't dealloc the elements ptr, as it isn't its own alloc
        let context = pg_sys::GetMemoryChunkContext(list as *mut _ as *mut _);
        if context == ptr::null_mut() {
            panic!("Context free list?");
        }
        let buf = pg_sys::MemoryContextAlloc(context, alloc_size);
        if buf == ptr::null_mut() {
            panic!("List allocation failure");
        }
        ptr::copy_nonoverlapping(list.elements, buf.cast(), list.length as _);
        // If the old buffer is pointers, we would like everyone dereferencing them to segfault,
        // if OIDs, Postgres will surface errors quickly on InvalidOid, etc.
        #[cfg(debug_assertions)]
        ptr::write_bytes(list.elements, 0x7F, list.length as _);
        list.elements = buf.cast();
    } else {
        // We already have a separate buf, making this easy.
        pg_sys::repalloc(list.elements.cast(), target * mem::size_of::<pg_sys::ListCell>());
    }

    list.max_length = target as _;
}

unsafe fn destroy_list(list: *mut pg_sys::List) {
    // The only question is if we have two allocations or one?
    if (*list).elements != ptr::addr_of_mut!((*list).initial_elements).cast() {
        pg_sys::pfree((*list).elements.cast());
    }
    pg_sys::pfree(list.cast());
}

#[derive(Debug)]
pub struct ListIter<T> {
    list: List<T>,
    iter: RawCellIter<T>,
}

/// A list being drained.
#[derive(Debug)]
pub struct Drain<'a, T> {
    /// Index of tail to preserve
    tail_start: u32,
    /// Length of tail
    tail_len: u32,
    /// Current remaining range to remove
    iter: RawCellIter<T>,
    origin: &'a mut List<T>,
    raw: *mut pg_sys::List,
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        if self.raw == ptr::null_mut() {
            return;
        }

        // SAFETY: The raw repr accepts null ptrs, but we just checked it's okay.
        unsafe {
            // Note that this may be 0, unlike elsewhere!
            let len = (*self.raw).length;
            if len == 0 && self.tail_len == 0 {
                // Can't simply leave it be due to Postgres List invariants, else it leaks
                destroy_list(self.raw)
            } else {
                // Need to weld over the drained part and fix the length
                let src = (*self.raw).elements.add(self.tail_start as _);
                let dst = (*self.raw).elements.add(len as _);
                ptr::copy(src, dst, self.tail_len as _); // may overlap
                (*self.raw).length = len + (self.tail_len as ffi::c_int);

                // Put it back now that all invariants have been repaired
                *self.origin = List::Cons(ListHead {
                    list: NonNull::new_unchecked(self.raw),
                    _type: PhantomData,
                });
            }
        }
    }
}

impl<T: Enlist> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Enlist> Iterator for ListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Enlist> IntoIterator for List<T> {
    type IntoIter = ListIter<T>;
    type Item = T;

    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len();
        let iter = match &mut self {
            List::Nil => Default::default(),
            List::Cons(head) => {
                let ptr = head.as_mut_cells_ptr();
                let end = unsafe { ptr.add(len) };
                RawCellIter { ptr, end }
            }
        };
        ListIter { list: self, iter }
    }
}

impl<T> Drop for ListIter<T> {
    fn drop(&mut self) {
        if let List::Cons(head) = &mut self.list {
            unsafe { destroy_list(head.list.as_ptr()) }
        }
    }
}

/// Needed because otherwise List hits incredibly irritating lifetime issues.
///
/// This must remain a private type, as casual usage of it is wildly unsound.
///
/// # Safety
/// None. Repent that you made this.
///
/// This atrocity assumes pointers passed in are valid or that ptr >= end.
#[derive(Debug, PartialEq)]
struct RawCellIter<T> {
    ptr: *mut ListCell<T>,
    end: *mut ListCell<T>,
}

impl<T> Default for RawCellIter<T> {
    fn default() -> Self {
        RawCellIter { ptr: ptr::null_mut(), end: ptr::null_mut() }
    }
}

impl<T: Enlist> Iterator for RawCellIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.ptr < self.end {
            let ptr = self.ptr;
            // SAFETY: It's assumed that the pointers are valid on construction
            unsafe {
                self.ptr = ptr.add(1);
                Some(T::apoptosis(ptr.cast()).read())
            }
        } else {
            None
        }
    }
}