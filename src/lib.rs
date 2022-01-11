#![cfg_attr(feature = "dox", feature(doc_notable_trait))]

use glib::subclass::SignalId;
use glib::subclass::signal::SignalBuilder;
use glib::translate::{ToGlibPtr, IntoGlib};
use glib::{SignalHandlerId, Quark, StaticType, Closure, BoolError};
use glib::{ObjectType, ObjectExt};
use glib::translate::from_glib;
use glib::value::FromValue;
use std::fmt::Debug;
use std::marker::PhantomData;

#[cfg(feature = "futures-channel")]
mod signal_stream;
#[cfg(feature = "futures-channel")]
pub use signal_stream::{SignalStream, OnceFuture, ConnectEof};

mod borrowed_object;
pub use borrowed_object::BorrowedObject;

mod pointer;
pub use pointer::Pointer;

mod value_option;
pub use value_option::{ToValueOption, PrimitiveValue};

mod from_values;
pub use from_values::FromValues;

mod macros;

#[doc(hidden)]
pub use glib; // for macro use

pub use glib::SignalFlags;

pub trait Signal: Copy + Debug {
	type Object: ObjectType;
	type Arguments: for<'a> FromValues<'a> + 'static;
	type Return: ToValueOption;

	const NAME: &'static str;
	const FLAGS: SignalFlags = SignalFlags::empty();

	fn signal() -> SignalId {
		SignalId::lookup(Self::NAME, <Self::Object as StaticType>::static_type())
			.expect(Self::NAME)
	}
}

pub trait DetailedSignal: Copy + Debug + Into<ConnectDetails<Self>> {
	type Signal: Signal;
	type Object: ObjectType;
	type Arguments: for<'a> FromValues<'a> + 'static;
	type Return: ToValueOption;

	const DETAIL: Option<&'static str>;

	fn detail() -> Option<Quark> {
		match Self::DETAIL {
			Some(detail) => Some(Quark::try_string(detail).expect(detail)),
			None => None,
		}
	}

	fn create_detail() -> Quark {
		Quark::from_string(Self::DETAIL.expect("detail string required"))
	}
}

impl<T: Signal> DetailedSignal for T {
	type Signal = Self;
	type Object = <Self::Signal as Signal>::Object;
	type Arguments = <Self::Signal as Signal>::Arguments;
	type Return = <Self::Signal as Signal>::Return;

	const DETAIL: Option<&'static str> = None;
}

pub trait BuildableSignal: Signal {
	fn builder<F: FnOnce(SignalBuilder) -> glib::subclass::Signal>(f: F) -> glib::subclass::Signal;
}

impl<T: Signal> BuildableSignal for T where
	<Self::Return as ToValueOption>::Type: StaticType
{
	fn builder<F: FnOnce(SignalBuilder) -> glib::subclass::Signal>(f: F) -> glib::subclass::Signal {
		let return_type = <<Self::Return as ToValueOption>::Type as StaticType>::static_type().into();
		let argument_types: Vec<_> = <Self::Arguments as FromValues>::static_types()
			.into_iter().map(From::from)
			.collect();
		let builder = glib::subclass::Signal::builder(Self::NAME, &argument_types, return_type)
			.flags(Self::FLAGS);
		f(builder)
	}
}

pub trait BuildSignal: BuildableSignal {
	fn build() -> glib::subclass::Signal {
		Self::builder(|b| b.build())
	}
}

#[cfg_attr(feature = "dox", doc(notable_trait))]
pub trait Notifies<T: Signal>: ObjectType {
}

#[derive(Copy, Clone, Debug)]
pub struct ConnectDetails<S = ()> {
	signal: SignalId,
	detail: Option<Quark>,
	pub run_after: bool,
	_signal: PhantomData<S>,
}

impl<S> ConnectDetails<S> {
	pub unsafe fn with_parts(signal: SignalId, detail: Option<Quark>, run_after: bool) -> Self {
		Self {
			signal,
			detail,
			run_after,
			_signal: PhantomData,
		}
	}

	pub fn normalize(&self) -> ConnectDetails<()> {
		unsafe {
			ConnectDetails::with_parts(self.signal(), self.detail(), self.run_after)
		}
	}

	pub fn signal(&self) -> SignalId {
		self.signal
	}

	pub fn detail(&self) -> Option<Quark> {
		self.detail
	}
}

impl<S: DetailedSignal> ConnectDetails<S> {
	pub fn new() -> Self {
		Self::with_after(false)
	}

	pub fn with_after(run_after: bool) -> Self {
		Self {
			signal: <S::Signal as Signal>::signal(),
			detail: S::detail(),
			run_after,
			_signal: PhantomData,
		}
	}

	pub fn set_detail(&mut self, detail: Quark) {
		assert!(self.detail.is_none());
		self.detail = Some(detail);
	}
}

impl<S: Signal> ConnectDetails<S> {
	pub fn with_detail<D: Into<Option<Quark>>>(detail: D) -> Self {
		Self {
			signal: S::signal(),
			detail: detail.into(),
			run_after: false,
			_signal: PhantomData,
		}
	}
}

impl ConnectDetails<()> {
	pub fn with_name<O: StaticType>(signal: &str, after: bool) -> Option<Self> {
		let (signal, detail) = SignalId::parse_name(signal, O::static_type(), false)?;
		let detail = match detail.into_glib() {
			// TODO: remove once https://github.com/gtk-rs/gtk-rs-core/issues/462 is resolved
			0 => None,
			_ => Some(detail),
		};
		Some(unsafe {
			Self::with_parts(signal, detail, after)
		})
	}
}

impl<S: DetailedSignal> From<ConnectDetails<S>> for ConnectDetails<()> {
	fn from(v: ConnectDetails<S>) -> Self {
		v.normalize()
	}
}

impl<S: DetailedSignal> From<S> for ConnectDetails<S> {
	fn from(_: S) -> Self {
		Self::new()
	}
}

pub trait ObjectSignalExt: ObjectType {
	unsafe fn handle_closure(&self, signal: &ConnectDetails, callback: &Closure) -> Result<SignalHandlerId, BoolError>;
	fn remove_handle(&self, handle: SignalHandlerId);

	fn handle<S, S_, C>(&self, signal: S_, callback: C) -> SignalHandlerId where
		C: Fn(&Self, S::Arguments) -> <S::Return as ToValueOption>::Type,
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>;

	#[cfg(feature = "futures-channel")]
	fn signal_stream<S, S_>(&self, signal: S_) -> SignalStream<Self, S::Arguments> where
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
		<S::Return as ToValueOption>::Type: Default;
}

impl<O: ObjectType> ObjectSignalExt for O where
	for<'a> BorrowedObject<'a, O>: FromValue<'a>,
{
	unsafe fn handle_closure(&self, signal: &ConnectDetails, callback: &Closure) -> Result<SignalHandlerId, BoolError> {
		let handle = glib::gobject_ffi::g_signal_connect_closure_by_id(
			self.as_object_ref().to_glib_none().0,
			signal.signal().into_glib(),
			signal.detail().map(|q| q.into_glib()).unwrap_or(0),
			callback.to_glib_none().0,
			signal.run_after.into_glib(),
		);
		match handle {
			0 => Err(glib::bool_error!("failed to connect signal {:?} of type {:?}", signal, Self::static_type())),
			handle => Ok(from_glib(handle)),
		}
	}

	fn handle<S, S_, C>(&self, signal: S_, callback: C) -> SignalHandlerId where
		C: Fn(&Self, S::Arguments) -> <S::Return as ToValueOption>::Type,
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
	{
		let signal = signal.into();
		unsafe {
			let callback = Closure::new_unsafe(move |values| {
				let (this, args) = values.split_first().unwrap();
				let this: BorrowedObject<Self> = this.get().unwrap();
				let args = S::Arguments::from_values(args).unwrap();
				callback(&this, args).into().to_value_option()
			});
			self.handle_closure(&signal.normalize(), &callback).unwrap()
		}
	}

	fn remove_handle(&self, handle: SignalHandlerId) {
		self.disconnect(handle)
	}

	#[cfg(feature = "futures-channel")]
	fn signal_stream<S, S_>(&self, signal: S_) -> SignalStream<Self, S::Arguments> where
		S: DetailedSignal,
		S_: Into<ConnectDetails<S>>,
		Self: Notifies<S::Signal>,
		<S::Return as ToValueOption>::Type: Default,
	{
		let signal = signal.into();
		SignalStream::connect(self, signal, |_, _| Default::default())
	}
}
