use glib::{Value, ObjectExt, WeakRef, ObjectType, Closure, SignalHandlerId, value::FromValue, g_warning};
use futures_core::{Stream, FusedStream, FusedFuture, ready};
use futures_channel::mpsc;
use std::hint::unreachable_unchecked;
use std::{fmt, io, ptr};
use std::error::Error;
use std::{pin::Pin, mem::ManuallyDrop};
use std::task::Poll;
use std::future::Future;

use crate::{ConnectDetails, FromValues, ToValueOption, BorrowedObject, DetailedSignal, ObjectSignalExt};

#[must_use]
#[cfg_attr(feature = "dox", doc(cfg(feature = "futures")))]
#[derive(Debug)]
pub struct SignalStream<O: ObjectType, T> {
	rx: mpsc::UnboundedReceiver<T>,
	target: WeakRef<O>,
	handle: Option<SignalHandlerId>,
}

impl<O: ObjectType, T> SignalStream<O, T> {
	pub fn connect<F, S>(target: &O, signal: ConnectDetails<S>, res: F) -> Self where
		S: DetailedSignal<Arguments=T>,
		T: for<'a> FromValues<'a> + 'static,
		F: Fn(&O, &T) -> <<S as DetailedSignal>::Return as ToValueOption>::Type + 'static,
		for<'a> BorrowedObject<'a, O>: FromValue<'a>,
	{
		let (tx, rx) = futures_channel::mpsc::unbounded();
		let callback = move |values: &[Value]| {
			let (this, args) = values.split_first().unwrap();
			let this: BorrowedObject<O> = this.get().unwrap();
			let args = FromValues::from_values(args).unwrap();
			let res = res(&this, &args);
			match tx.unbounded_send(args) {
				Ok(()) => (),
				Err(e) => {
					g_warning!("glib-signal", "Failed to signal {:?}: {:?}", signal, e);
				},
			}
			res.into().to_value_option()
		};
		let handle = unsafe {
			target.handle_closure(&signal.normalize(), &Closure::new_unsafe(callback))
		}.unwrap();

		SignalStream {
			rx,
			target: target.downgrade(),
			handle: Some(handle),
		}
	}

	pub fn once(self) -> OnceFuture<O, T> {
		OnceFuture::new(self)
	}

	pub fn disconnect(&mut self) {
		if let Some(handle) = self.handle.take() {
			if let Some(target) = self.target.upgrade() {
				target.disconnect(handle);
			}
		}
	}

	pub fn into_target(self) -> WeakRef<O> {
		let mut this = ManuallyDrop::new(self);
		this.disconnect();
		unsafe {
			ptr::read(&this.target)
		}
	}

	pub fn target(&self) -> &WeakRef<O> {
		&self.target
	}

	pub fn attach_target(self) -> SignalStreamSelf<O, T> {
		SignalStreamSelf::from(self)
	}
}

impl<O: ObjectType, T> Stream for SignalStream<O, T> {
	type Item = T;

	fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
		let rx = unsafe { self.map_unchecked_mut(|s| &mut s.rx) };
		rx.poll_next(cx)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.rx.size_hint()
	}
}

impl<O: ObjectType, T> FusedStream for SignalStream<O, T> {
	fn is_terminated(&self) -> bool {
		self.rx.is_terminated()
	}
}

impl<O: ObjectType, T> Drop for SignalStream<O, T> {
	fn drop(&mut self) {
		self.disconnect();
	}
}

#[derive(Debug, Copy, Clone)]
pub struct ConnectEof;

impl fmt::Display for ConnectEof {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "unexpected connect handle EOF")
	}
}

impl Error for ConnectEof { }

impl From<ConnectEof> for io::Error {
	fn from(eof: ConnectEof) -> Self {
		io::Error::new(io::ErrorKind::UnexpectedEof, eof)
	}
}

impl From<ConnectEof> for glib::Error {
	fn from(eof: ConnectEof) -> Self {
		glib::Error::new(glib::FileError::Pipe, &format!("{:?}", eof))
	}
}

pub struct OnceFuture<O: ObjectType, T> {
	stream: Option<SignalStream<O, T>>,
}

impl<O: ObjectType, T> OnceFuture<O, T> {
	pub fn new(stream: SignalStream<O, T>) -> Self {
		Self {
			stream: Some(stream),
		}
	}

	/// check `is_terminated` first!
	pub fn into_inner(self) -> SignalStream<O, T> {
		self.stream.unwrap()
	}
}

impl<O: ObjectType, T> Future for OnceFuture<O, T> {
	type Output = Result<(T, WeakRef<O>), ConnectEof>;

	fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		//let this = unsafe { self.get_unchecked_mut() };
		let (res, stream) = unsafe {
			let mut stream = self.map_unchecked_mut(|this| &mut this.stream);
			let res = match stream.as_mut().as_pin_mut() {
				Some(stream) => ready!(stream.poll_next(cx)),
				None => return Poll::Pending,
			};
			(res, match stream.get_unchecked_mut().take() {
				Some(s) => s,
				None => unreachable_unchecked(),
			})
		};
		let obj = stream.into_target();
		Poll::Ready(match res {
			Some(res) => Ok((res, obj)),
			None => Err(ConnectEof),
		})
	}
}

impl<O: ObjectType, T> FusedFuture for OnceFuture<O, T> {
	fn is_terminated(&self) -> bool {
		self.stream.as_ref().map(|s| s.is_terminated()).unwrap_or(true)
	}
}

pub struct SignalStreamSelf<O: ObjectType, T> {
	inner: SignalStream<O, T>,
}

impl<O: ObjectType, T> From<SignalStream<O, T>> for SignalStreamSelf<O, T> {
	fn from(inner: SignalStream<O, T>) -> Self {
		Self {
			inner,
		}
	}
}

impl<O: ObjectType, T> Stream for SignalStreamSelf<O, T> {
	type Item = (Option<O>, T);

	fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
		let mut inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
		Poll::Ready(ready!(inner.as_mut().poll_next(cx))
			.map(|res| (inner.target().upgrade(), res))
		)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}
}

impl<O: ObjectType, T> FusedStream for SignalStreamSelf<O, T> {
	fn is_terminated(&self) -> bool {
		self.inner.is_terminated()
	}
}
