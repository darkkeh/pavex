use crate::compiler::component::{
    Constructor, ErrorHandler, ErrorObserver, RequestHandler, WrappingMiddleware,
};
use crate::compiler::computation::Computation;
use crate::language::ResolvedType;
use std::borrow::Cow;

/// A transformation that, given a set of inputs, **constructs** a new type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum HydratedComponent<'a> {
    Constructor(Constructor<'a>),
    RequestHandler(RequestHandler<'a>),
    WrappingMiddleware(WrappingMiddleware<'a>),
    ErrorHandler(Cow<'a, ErrorHandler>),
    Transformer(Computation<'a>),
    ErrorObserver(ErrorObserver<'a>),
}

impl<'a> HydratedComponent<'a> {
    pub(crate) fn input_types(&self) -> Cow<[ResolvedType]> {
        match self {
            HydratedComponent::Constructor(c) => c.input_types(),
            HydratedComponent::RequestHandler(r) => Cow::Borrowed(r.input_types()),
            HydratedComponent::ErrorHandler(e) => Cow::Borrowed(e.input_types()),
            HydratedComponent::Transformer(c) => c.input_types(),
            HydratedComponent::WrappingMiddleware(c) => Cow::Borrowed(c.input_types()),
            HydratedComponent::ErrorObserver(eo) => Cow::Borrowed(eo.input_types()),
        }
    }

    pub(crate) fn output_type(&self) -> Option<&ResolvedType> {
        match self {
            HydratedComponent::Constructor(c) => Some(c.output_type()),
            HydratedComponent::RequestHandler(r) => Some(r.output_type()),
            HydratedComponent::ErrorHandler(e) => Some(e.output_type()),
            HydratedComponent::WrappingMiddleware(e) => Some(e.output_type()),
            // TODO: we are not enforcing that the output type of a transformer is not
            //  the unit type. In particular, you can successfully register a `Result<T, ()>`
            //  type, which will result into a `MatchResult` with output `()` for the error.
            HydratedComponent::Transformer(c) => c.output_type(),
            HydratedComponent::ErrorObserver(_) => None,
        }
    }

    pub(crate) fn is_fallible(&self) -> bool {
        self.output_type().map_or(false, |t| t.is_result())
    }

    /// Returns a [`Computation`] that matches the transformation carried out by this component.
    pub(crate) fn computation(&self) -> Computation<'a> {
        match self {
            HydratedComponent::Constructor(c) => c.0.clone(),
            HydratedComponent::RequestHandler(r) => r.callable.clone().into(),
            HydratedComponent::WrappingMiddleware(w) => w.callable.clone().into(),
            HydratedComponent::ErrorHandler(e) => e.callable.clone().into(),
            HydratedComponent::Transformer(t) => t.clone(),
            HydratedComponent::ErrorObserver(eo) => eo.callable.clone().into(),
        }
    }

    pub(crate) fn into_owned(self) -> HydratedComponent<'static> {
        match self {
            HydratedComponent::Constructor(c) => HydratedComponent::Constructor(c.into_owned()),
            HydratedComponent::RequestHandler(r) => {
                HydratedComponent::RequestHandler(r.into_owned())
            }
            HydratedComponent::WrappingMiddleware(w) => {
                HydratedComponent::WrappingMiddleware(w.into_owned())
            }
            HydratedComponent::ErrorHandler(e) => {
                HydratedComponent::ErrorHandler(Cow::Owned(e.into_owned()))
            }
            HydratedComponent::Transformer(t) => HydratedComponent::Transformer(t.into_owned()),
            HydratedComponent::ErrorObserver(eo) => {
                HydratedComponent::ErrorObserver(eo.into_owned())
            }
        }
    }

    pub(crate) fn is_error_handler(&self) -> bool {
        matches!(self, HydratedComponent::ErrorHandler(_))
    }
}
