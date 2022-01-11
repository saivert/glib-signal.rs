use glib::{Value, StaticType, ObjectType, Type};
use glib::value::FromValue;
use glib::translate::{ToGlibPtr, FromGlibPtrBorrow, from_glib_borrow};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;

#[derive(Debug)]
#[repr(transparent)]
pub struct BorrowedObject<'a, O> {
	inner: ManuallyDrop<O>,
	_borrow: PhantomData<&'a O>,
}

impl<'a, O> BorrowedObject<'a, O> {
	pub fn forget(inner: O) -> Self {
		Self {
			inner: ManuallyDrop::new(inner),
			_borrow: PhantomData,
		}
	}

	/// If `O` lives beyond `'a`, bad things may happen.
	pub unsafe fn into_inner(self) -> ManuallyDrop<O> {
		self.inner
	}
}

impl<'a, O: ObjectType> BorrowedObject<'a, O> {
	/// Copy this reference
	pub fn copy_ref(&self) -> Self {
		unsafe {
			std::ptr::read(self)
		}
	}
}

impl<'a, O> Deref for BorrowedObject<'a, O> {
	type Target = O;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a, O> AsRef<O> for BorrowedObject<'a, O> {
	fn as_ref(&self) -> &O {
		&self.inner
	}
}

unsafe impl<'a, O: ObjectType + FromGlibPtrBorrow<*mut O::GlibType>> FromValue<'a> for BorrowedObject<'a, O> {
	type Checker = glib::value::GenericValueTypeChecker<O>;

	unsafe fn from_value(value: &'a Value) -> Self {
		let value = value.to_glib_none();
		let borrowed: glib::translate::Borrowed<O> = from_glib_borrow(glib::gobject_ffi::g_value_get_object(value.0) as *mut O::GlibType);
		Self::forget(borrowed.into_inner())
	}
}

impl<'a, O: StaticType> StaticType for BorrowedObject<'a, O> {
	fn static_type() -> Type {
		O::static_type()
	}
}
