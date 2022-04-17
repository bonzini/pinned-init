//! This library is used to facilitate safe pinned initialization ([what is this?](#what-problem-this-library-solves)) for as many
//! parties involved as possible.
//!
//! Jump to the relevant section:
//! - [initialization](#initialization-of-a-type-supported-by-this-library) of a type supported by this library.
//! - [declaration](#declaration-of-a-type-with-field-types-supported-by-this-library) of a type with field types supported by this library.
//! - [declaration](#declaration-of-a-type-with-field-types-not-supported-by-this-library) of a type with fields **not** supported by this library.
//! - [implementing](#implementing-support-for-a-custom-pointer-type) support for a custom pointer type.
//!
//! Using this library requires a nightly version of rust, until the following
//! features are stablized:
//! - generic_associated_types
//! - const_ptr_offset_from
//! - const_refs_to_cell
//! Users of the proc maco attributes [`manual_init`] and [`pinned_init`] will
//! need to also add these features to their crate, because the macros generate
//! code using these features.
//!
//! # Initialization of a type supported by this library
//! Lets say you use a crate defining a self referential data type named
//! `PtrBuf` with support for this library.
//!
//! Because this section is only about how we initialize such a type I will not
//! show any implementation details. Instead I will use standard rust types to
//! show what these type are conceptually made of. If you are interested how
//! such a type actually is created, feel free to head to that [section](#declaration-of-a-type-with-field-types-supported-by-this-library).
//!
//! ## What `PtrBuf` defines
//!
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! use pinned_init::prelude::*;
//! #[manual_init]
//! pub struct PtrBuf<T> {
//!     idx: *const T,
//!     buf: [T; 64],
//! }
//! ```
//! To create a `PtrBuf` the crate provides the following implementation:
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use pinned_init::prelude::*;
//! # #[manual_init]
//! # pub struct PtrBuf<T> {
//! #     idx: *const T,
//! #     buf: [T; 64],
//! # }
//! impl<T> From<[T; 64]> for PtrBufUninit<T> {
//!     fn from(arr: [T; 64]) -> Self { todo!() }
//! }
//! ```
//! The `PtrBufUninit` type is automatically generated by this library (invoked
//! via the see [`manual_init`] proc macro) and represents a partially
//! uninitialized, version of a `PtrBuf`.
//!
//! To use a `PtrBuf`, a next function is defined:
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use pinned_init::prelude::*;
//! # use core::pin::Pin;
//! # #[manual_init]
//! # pub struct PtrBuf<T> {
//! #     idx: *const T,
//! #     buf: [T; 64],
//! # }
//! impl<T> PtrBuf<T> {
//!     pub fn next(self: Pin<&mut Self>) -> Option<&T> { todo!() }
//! }
//! ```
//!
//! ## Creating a `PtrBuf`
//!
//! To begin the creation process of a `PtrBuf` we first must create a
//! `PtrBufUninit`:
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use pinned_init::prelude::*;
//! # use core::pin::Pin;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #     buf: [T; 64],
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(arr: [T; 64]) -> Self {
//! #         Self { buf: arr }
//! #     }
//! # }
//! let uninit: PtrBufUninit<i32> = PtrBufUninit::from([42; 64]);
//! ```
//! Because this library facilitates pinned initialization, we now need to find
//! some stable memory for our `uninit`, that we can allocate for at least the
//! duration of the `PtrBuf` we intend to use.
//!
//! In this Example we will use [`Box`], but you could use any other mechanism
//! to store your object, as long as it lives long enough and is supported by
//! this library (see [this section](#implementing-support-for-a-custom-pointer-type),
//! if you want to know what this means).
//!
//! Now we need to call [`Box::pin`] to create an already pinned pointer to our
//! data, if your pointer type supports it you could also create it first and
//! then pin it later, but in many cases this is not necessary.
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use core::pin::Pin;
//! # use pinned_init::prelude::*;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #     buf: [T; 64],
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(arr: [T; 64]) -> Self {
//! #         Self { buf: arr }
//! #     }
//! # }
//! # let uninit: PtrBufUninit<i32> = PtrBufUninit::from([42; 64]);
//! let boxed: Pin<Box<PtrBufUninit<i32>>> = Box::pin(uninit);
//! ```
//!
//! Now all that is left to do is to call [`SafePinnedInit::init`], this trait
//! is implemented for [`Pin<P>`] when `P` is pointer with support for this
//! library.
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use core::pin::Pin;
//! use pinned_init::prelude::*;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #     buf: [T; 64],
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(arr: [T; 64]) -> Self {
//! #         Self { buf: arr }
//! #     }
//! # }
//! # let uninit: PtrBufUninit<i32> = PtrBufUninit::from([42; 64]);
//! # let boxed: Pin<Box<PtrBufUninit<i32>>> = Box::pin(uninit);
//! let init: Pin<Box<PtrBuf<i32>>> = boxed.init();
//! ```
//!
//! Full code:
//! ```rust
//! #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use core::pin::Pin;
//! use pinned_init::prelude::*;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #     buf: [T; 64],
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(arr: [T; 64]) -> Self {
//! #         Self { buf: arr }
//! #     }
//! # }
//! let uninit: PtrBufUninit<i32> = PtrBufUninit::from([42; 64]);
//! let boxed: Pin<Box<PtrBufUninit<i32>>> = Box::pin(uninit);
//! let init: Pin<Box<PtrBuf<i32>>> = boxed.init();
//! ```
//!
//! # Declaration of a type with field types supported by this library
//! This involves writing no unsafe code yourself and is done by adding
//! [`pinned_init`] as an attribute to your struct and marking each field, that
//! requires initialization with `#[init]`:
//! ```rust
//! #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! use pinned_init::prelude::*;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #   buf: [T; 64],
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(buf: [T; 64]) -> Self {
//! #         Self {
//! #             buf
//! #         }
//! #     }
//! # }
//! #[pinned_init]
//! pub struct WithPtrBuf<'a, T> {
//!     #[init]
//!     msgs: PtrBuf<&'a str>,
//!     #[init]
//!     values: PtrBuf<T>,
//!     info: String,
//! }
//!
//! impl<'a, T> WithPtrBufUninit<'a, T> {
//!     pub fn new(info: String, msgs: [&'a str; 64], values: [T; 64]) -> Self {
//!         Self {
//!             msgs: msgs.into(),
//!             values: values.into(),
//!             info,
//!         }
//!     }
//! }
//! ```
//! And that is it.
//!
//! When you want to use a field, use the same API when using [`pin_project`]:
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use pinned_init::prelude::*;
//! # use core::pin::Pin;
//! # #[pinned_init]
//! # struct PtrBuf<T> {
//! #   buf: [T; 64],
//! # }
//! # impl<T> PtrBuf<T> {
//! #     pub fn next(self: Pin<&mut Self>) -> Option<&T> { todo!() }
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(buf: [T; 64]) -> Self {
//! #         Self {
//! #             buf
//! #         }
//! #     }
//! # }
//! #[pinned_init]
//! pub struct WithPtrBuf<'a, T> {
//!     #[init]
//!     msgs: PtrBuf<&'a str>,
//!     #[init]
//!     values: PtrBuf<T>,
//!     info: String,
//! }
//! impl<'a, T> WithPtrBuf<'a, T> {
//!     pub fn print_msgs(self: Pin<&mut Self>) {
//!         let mut this = self.project();
//!         while let Some(msg) = this.msgs.as_mut().next() {
//!             println!("{msg}");
//!         }
//!     }
//! }
//! ```
//!
//! # Declaration of a type with field types not supported by this library
//!
//! When using a field with a type which is not supported (for example an
//! uninitialized type or something that requires custom logic) you will not be
//! able to use [`pinned_init`]. Instead you will use [`manual_init`] and
//! additionally manually implement [`PinnedInit`]. The ussage of [`manual_init`]
//! is similar to [`pinned_init`], you mark fields that need initialization with
//! `#[init]`. However you need to specify `#[pin]` manually, if you want that
//! field to be structually pinned. When you want to initalize a field only
//! after pinning and that fields type does not implement [`PinnedInit`], you
//! need to use [`StaticUninit<T, INIT>`] here the `PtrBuf` example:
//! ```rust
//! #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! use pinned_init::prelude::*;
//! #[manual_init]
//! pub struct PtrBuf<T> {
//!     buf: [T; 64],
//!     #[init]
//!     ptr: StaticUninit<*const T>,
//! }
//! ```
//! Now we also need to implement a way to construct a `PtrBufUninit` and
//! implement PinnedInit:
//! ```rust
//! #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! use pinned_init::prelude::*;
//! # #[manual_init]
//! # pub struct PtrBuf<T> {
//! #     buf: [T; 64],
//! #     #[init]
//! #     ptr: StaticUninit<*const T>,
//! #     #[init]
//! #     end: StaticUninit<*const T>,
//! # }
//! impl<T> PinnedInit for PtrBufUninit<T> {
//!     type Initialized = PtrBuf<T>;
//!
//!     fn init_raw(this: NeedsPinnedInit<Self>) {
//!         let PtrBufOngoingInit {
//!             ptr,
//!             buf,
//!             end,
//!         } = this.begin_init();
//!         ptr.init(&*buf as *const T);
//!         end.init(buf.last().unwrap() as *const T);
//!     }
//! }
//!
//! impl<T> From<[T; 64]> for PtrBufUninit<T> {
//!     fn from(buf: [T; 64]) -> Self {
//!         Self {
//!             buf,
//!             ptr: StaticUninit::uninit(),
//!             end: StaticUninit::uninit(),
//!         }
//!     }
//! }
//! ```
//! Now implementing the `next` method is rather staight forward:
//! ```rust
//! # #![feature(generic_associated_types, const_ptr_offset_from, const_refs_to_cell)]
//! # use pinned_init::prelude::*;
//! # use core::pin::Pin;
//! # #[manual_init]
//! # pub struct PtrBuf<T> {
//! #     buf: [T; 64],
//! #     #[init]
//! #     ptr: StaticUninit<*const T>,
//! #     #[init]
//! #     end: StaticUninit<*const T>,
//! # }
//! # impl<T> PinnedInit for PtrBufUninit<T> {
//! #     type Initialized = PtrBuf<T>;
//! #     fn init_raw(this: NeedsPinnedInit<Self>) {
//! #         let PtrBufOngoingInit {
//! #             ptr,
//! #             buf,
//! #             end,
//! #         } = this.begin_init();
//! #         ptr.init(&*buf as *const T);
//! #         end.init(buf.last().unwrap() as *const T);
//! #     }
//! # }
//! # impl<T> From<[T; 64]> for PtrBufUninit<T> {
//! #     fn from(buf: [T; 64]) -> Self {
//! #         Self {
//! #             buf,
//! #             ptr: StaticUninit::uninit(),
//! #             end: StaticUninit::uninit(),
//! #         }
//! #     }
//! # }
//! impl<T> PtrBuf<T> {
//!     pub fn next(self: Pin<&mut Self>) -> Option<&T> {
//!         let this = self.project();
//!         if **this.ptr > **this.end {
//!             None
//!         } else {
//!             let res = unsafe {
//!                 // SAFETY: we were correctly initialized and checked bounds
//!                 // so this.ptr points to somewhere in buf.
//!                 &***this.ptr
//!             };
//!             **this.ptr = unsafe {
//!                 // SAFETY: the resulting pointer is either one byte after buf, or
//!                 // inside buf.
//!                 // An offset of 1 cannot overflow, because we allocated `[T;
//!                 // 64]` before. the allocation also does not wrap around the
//!                 // address space.
//!                 this.ptr.offset(1)
//!             };
//!             Some(res)
//!         }
//!     }
//! }
//! ```
//!
//! # Implementing support for a custom pointer type
//!
//! For a pointer to be supported by this library, it needs to implement
//! [`OwnedUniquePtr<T>`].
//!
//! This example shows the [`OwnedUniquePtr`] implementation for [`Box<T>`]:
//! ```rust,ignore
//! #![feature(generic_associated_types)]
//! use pinned_init::{ptr::OwnedUniquePtr, transmute::TransmuteInto};
//! use core::{marker::PhantomData, ops::{Deref, DerefMut}, pin::Pin};
//!
//! // SAFETY:
//! // - Box owns its data.
//! // - Box knows statically to have the only pointer that points to its
//! // value.
//! // - we provided the same pointer type for `Self::Ptr`.
//! unsafe impl<T: ?Sized> OwnedUniquePtr<T> for Box<T> {
//!     type Ptr<U: ?Sized> = Box<U>;
//!
//!     unsafe fn transmute_pointee_pinned<U>(this: Pin<Self>) ->
//!         Pin<Self::Ptr<U>>
//!     where
//!         T: TransmuteInto<U>,
//!     {
//!         unsafe {
//!             // SAFETY: we later repin the pointer and in between never move
//!             // the data behind it.
//!             let this = Pin::into_inner_unchecked(this);
//!             // this is safe, due to the requriements of this function
//!             let this: Box<U> = Box::from_raw(Box::into_raw(this));
//!             Pin::new_unchecked(this)
//!         }
//!     }
//! }
//! ```
//! # What problem this library solves
//!
//! Normally when writing rust, the principle of [RAII]() is used.
//! This is fine for most applications, but when one needs to already be pinned
//! for initalization to start, this becomes a problem.
//! Without this library you would need to resort to `unsafe` for all such
//! initializations.
//!
//! For example the `PtrBuf` from above without this library and without
//! [pin_project] could look like this:
//! ```rust
//! use core::{mem::MaybeUninit, pin::Pin, ptr};
//!
//! pub struct PtrBuf<T> {
//!     buf: [T; 64],
//!     ptr: *const T,
//!     end: *const T,
//! }
//!
//! impl<T> PtrBuf<T> {
//!     /// Construct a new PtrBuf.
//!     ///
//!     /// # Safety
//!     ///
//!     /// The caller needs to call [`PtrBuf::init`] before using this PtrBuf
//!     pub unsafe fn new(buf: [T; 64]) -> Self {
//!         Self {
//!             buf,
//!             ptr: ptr::null(),
//!             end: ptr::null(),
//!         }
//!     }
//!
//!     /// Initializes this PtrBuf
//!     ///
//!     /// # Safety
//!     ///
//!     /// The caller needs to guarantee that this function is only called
//!     /// once.
//!     pub unsafe fn init(self: Pin<&mut Self>) {
//!         let ptr = &self.buf as *const T;
//!         unsafe {
//!             // SAFETY: we do not move the data behind this pointer.
//!             self.get_unchecked_mut().ptr.write(ptr);
//!         }
//!     }
//!
//!     /// Fetches the next value, if present.
//!     ///
//!     /// # Safety
//!     /// The caller needs to call [`PtrBuf::init`] before calling this
//!     /// function.
//!     pub unsafe fn next(self: Pin<&mut Self>) -> Option<&T> {
//!         let this = unsafe {
//!             // SAFETY: We never move out of this pointer
//!             self.get_unchecked_mut()
//!         };
//!         debug_assert!(!ptr.is_null());
//!         if this.ptr > this.end {
//!             None
//!         } else {
//!             let res = unsafe {
//!                 // SAFETY: we checked bounds before and the caller
//!                 // guarantees, that they called `init`.
//!                 &*this.ptr
//!             };
//!             // SAFETY: the resulting pointer is either one byte after buf, or
//!             // inside buf.
//!             // An offset of 1 cannot overflow, because we allocated `[T;
//!             // 64]` before. the allocation also does not wrap around the
//!             // address space.
//!             this.ptr = this.ptr.offset(1);
//!             Some(res)
//!         }
//!     }
//! }
//! ```
//! Ugh.
//!
//! The worst thing about this way of performing pinned initialization is, that
//! users of the `PtrBuf` library will have to use unsafe to initialize and
//! use `PtrBuf`. It is very easy to forget a call to `init` if one creates a
//! `PtrBuf` and only later pins it. When the types are designed to be both used
//! before and after pinning, then this becomes even more of a problem source.
//!
//! Using unsafe for these invariants just results in rust code that is arguably
//! less ergonomic the same code in C.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![deny(unsafe_op_in_unsafe_fn, missing_docs)]
use crate::{
    needs_init::NeedsPinnedInit, private::BeginInit, ptr::OwnedUniquePtr, transmute::TransmuteInto,
};
use core::pin::Pin;

#[cfg(feature = "alloc")]
extern crate alloc;

macro_rules! if_cfg {
    (if $cfg:tt {$($body:tt)*} else {$($els:tt)*}) => {
        #[cfg $cfg]
        {
            $($body)*
        }
        #[cfg(not $cfg)]
        {
            $($els)*
        }
    };
}

pub mod needs_init;
pub mod ptr;
pub mod static_uninit;

pub use pinned_init_macro::{manual_init, pinned_init};

#[doc(hidden)]
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        manual_init,
        needs_init::{NeedsInit, NeedsPinnedInit},
        pinned_init,
        static_uninit::StaticUninit,
        PinnedInit, SafePinnedInit,
    };
}

#[doc(hidden)]
pub mod __private {
    pub use pin_project::pin_project;
}

#[doc(hidden)]
pub mod private {
    use core::pin::Pin;

    /// Trait implemented by the [`pinned_init`] and the [`manual_init`] proc
    /// macros. This trait should not be implemented manually.
    pub trait BeginInit {
        #[doc(hidden)]
        type OngoingInit<'init>: 'init
        where
            Self: 'init;
        #[doc(hidden)]
        unsafe fn __begin_init<'init>(self: Pin<&'init mut Self>) -> Self::OngoingInit<'init>
        where
            Self: 'init;
    }
}

/// Initializing a value in place (because it is pinned) requires a
/// transmuation. Because transmuations are inherently unsafe, this module aims
/// to create a safer abstraction and requires users to explicitly opt in to use
/// this initialization.
pub mod transmute {
    use core::{mem, pin::Pin};

    /// Marks and allows easier unsafe transmutation between types.
    /// This trait should **not** be implemented manually.
    ///
    /// When implementing this type manually (despite being told not to!), you
    /// must ensure, that `Self` is indeed transmutible to `T`, review the
    /// current [unsafe code guidelines]()
    /// to ensure that `Self` and `T` will also be transmutible in future
    /// versions of the compiler.
    ///
    /// When you use the proc macro attributes [`pinned_init`] and
    /// [`manual_init`] this trait will be implemented automatically to
    /// transmute from the uninitialized to the initialized variant.
    /// This is accompanied with static compile checks, that the layout of both
    /// types is the same.
    ///
    /// When using this trait, it is not required to use one of the provided
    /// functions, you may [`mem::transmute`] a value of `Self` to `T`, you may
    /// use a union to reinterpret a `Self` as a `T` and you may use pointer
    /// casts to cast a pointer to `Self` to a pointer to `T`.
    /// Of course you will need to still ensure the safety requirements of this
    /// trait.
    ///
    /// # Safety
    ///
    /// When implementing this trait the following conditions must be true:
    /// - `T` and `Self` have the same layout, compiler version updates must not
    /// break this invariant.
    /// - transmutation is only sound, if all invariants of `T` are satisfied by
    /// all values of `Self` transmuted with this trait.
    ///
    /// Again: **DO NOT IMPLEMENT THIS MANUALLY** as this requires very strict
    /// invariants to be upheld, that concern layout of types. The behaviour of
    /// the compiler has not yet been fully specified, so you need to take extra
    /// care to ensure future compiler compatiblity.
    pub unsafe trait TransmuteInto<T>: Sized {
        /// Unsafely transmutes `self` to `T`.
        ///
        /// # Safety
        ///
        /// - `T` and `Self` must have the same layout.
        /// - All invariants of `T` need to be satisfied by `self`.
        #[inline]
        unsafe fn transmute(self) -> T {
            unsafe {
                let ptr = &self as *const Self;
                mem::forget(self);
                let ptr = Self::transmute_ptr(ptr);
                ptr.read()
            }
        }

        /// Unsafely transmutes a pointer to `Self` to a pointer to `T`.
        ///
        /// # Safety
        ///
        /// - `T` and `Self` must have the same layout.
        /// - All invariants of `T` need to be satisfied by the value at `this`.
        unsafe fn transmute_ptr(this: *const Self) -> *const T;
    }

    // SAFETY: [`Pin`] is `repr(transparent)`, thus permitting transmutations between
    // `Pin<P> <-> P`. Because T permits transmuting `T -> U`, transmuting
    // `Pin<T> -> Pin<U>` is also permitted (effectively `Pin<T> -> T -> U
    // ->`Pin<U>`).
    unsafe impl<T, U> TransmuteInto<Pin<U>> for Pin<T>
    where
        T: TransmuteInto<U>,
    {
        #[inline]
        unsafe fn transmute_ptr(this: *const Self) -> *const Pin<U> {
            unsafe {
                // SAFETY: `T: TransmuteInto<U>` guarantees that we can
                // transmute `T -> U`. The caller needs to guarantee, that the
                // invariants of `U` are upheld when this `T` is transmuted to
                // `U`.
                mem::transmute(this)
            }
        }
    }
}

/// Facilitates pinned initialization.
/// Before you implement this trait manually, look at the [`pinned_init`] proc
/// macro attribute, it can be used to implement this trait in a safe and sound
/// fashion in many cases.
///
/// You will need to implement this trait yourself, if your struct contains any
/// fields with the [`static_uninit::StaticUninit`] type. When implementing this
/// trait manually, use the [`manual_init`] proc macro attribute to implement
/// [`BeginInit`] for your struct, as implementing that trait is not supposed to
/// be done manually.
pub trait PinnedInit: TransmuteInto<Self::Initialized> + BeginInit {
    /// The initialized version of `Self`. `Self` can be transmuted via
    /// [`TransmuteInto`] into this type.
    type Initialized;

    /// Initialize the value behind the given pointer, this pointer ensures,
    /// that `Self` really will be initialized.
    fn init_raw(this: NeedsPinnedInit<Self>);
}

// used to prevent accidental/mailicious implementations of `SafePinnedInit`
mod sealed {
    use super::*;

    pub trait Sealed<T: PinnedInit> {}

    impl<T: PinnedInit, P: OwnedUniquePtr<T>> Sealed<T> for Pin<P> {}
}

/// Sealed trait to facilitate safe initialization of the types supported by
/// this crate.
///
/// Use this traits [`Self::init`] method to initialize the T contained in `self`.
/// This trait is implemented only for [`Pin<P>`] `where P:` [`OwnedUniquePtr<T>`] `, T:` [`PinnedInit`].
pub trait SafePinnedInit<T: PinnedInit>: sealed::Sealed<T> + Sized {
    /// The type that represents the initialized version of `self`.
    type Initialized;

    /// Initialize the contents of `self`.
    fn init(self) -> Self::Initialized;
}

impl<T: PinnedInit, P: OwnedUniquePtr<T>> SafePinnedInit<T> for Pin<P> {
    type Initialized = Pin<P::Ptr<T::Initialized>>;

    #[inline]
    fn init(mut self) -> Self::Initialized {
        unsafe {
            // SAFETY: `self` implements `OwnedUniquePtr`, thus giving us unique
            // access to the data behind `self`. Because we call `T::init_raw`
            // and `P::transmute_pointee_pinned` below, the contract of
            // `NeedsPinnedInit::new_unchecked` is fullfilled (all pointers to
            // the data are aware of the new_unchecked call).
            let this = NeedsPinnedInit::new_unchecked(self.as_mut());
            T::init_raw(this);
            P::transmute_pointee_pinned(self)
        }
    }
}
