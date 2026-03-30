/// Generate boilerplate mapping code between “entity”/DTO types.
///
/// `map_entity!` helps you define small, explicit transformations from one type to another
/// by generating either:
/// - an implementation of `From<Source> for Target`, or
/// - a `Target::from_source(...)` constructor, or
/// - a `Target::from_parts(...)` constructor that combines multiple inputs.
///
/// This is useful for mapping:
/// - API requests → domain entities
/// - domain entities → API responses
/// - database models → domain models
/// - multiple inputs (request + persisted entity) → a composed output
///
/// ## Notes
///
/// - The expressions you provide for each field are inserted verbatim into the generated impl.
/// - In versions 2 and 3, the mapping function is generated inside an `impl Target { ... }` block.
/// - The macro does **not** validate field names; if you specify a field that does not exist on
///   the target type, you will get a compiler error.
/// - If you reference fields from a source value, make sure you reference the correct variable
///   name (`source` in version 1, the identifier you chose in version 2, or the identifiers you
///   chose in version 3).
///
/// ## Version 1: `From<Source> for Target`
///
/// Syntax:
///
/// ```rust,ignore
/// map_entity!(SourceType => TargetType {
///     field_a: /* expr using `source` */,
///     field_b: /* expr using `source` */,
/// });
/// ```
///
/// This generates:
///
/// ```rust,ignore
/// impl From<SourceType> for TargetType {
///     fn from(source: SourceType) -> Self {
///         Self { field_a: ..., field_b: ... }
///     }
/// }
/// ```
///
/// ### Example
///
/// ```rust,ignore
/// #[derive(Clone)]
/// struct RegisterUserRequest {
///     first_name: String,
///     last_name: String,
///     email: String,
/// }
///
/// #[derive(Debug)]
/// struct User {
///     first_name: String,
///     last_name: String,
///     email: String,
/// }
///
/// map_entity!(RegisterUserRequest => User {
///     first_name: source.first_name,
///     last_name: source.last_name,
///     email: source.email,
/// });
///
/// // usage:
/// let req = RegisterUserRequest {
///     first_name: "Ada".into(),
///     last_name: "Lovelace".into(),
///     email: "ada@example.com".into(),
/// };
/// let user: User = req.into();
/// ```
///
/// ## Version 2: `Target::from_source(source)`
///
/// Syntax:
///
/// ```rust,ignore
/// map_entity!(TargetType {
///     from source: SourceType => {
///         field_a: /* expr using `source` */,
///         field_b: /* expr using `source` */,
///     }
/// });
/// ```
///
/// This generates:
///
/// ```rust,ignore
/// impl TargetType {
///     pub fn from_source(source: SourceType) -> Self {
///         Self { field_a: ..., field_b: ... }
///     }
/// }
/// ```
///
/// ### Example
///
/// ```rust,ignore
/// #[derive(Clone)]
/// struct User {
///     id: u32,
///     email: String,
/// }
///
/// struct RegisterUserResponse {
///     id: u32,
///     email: String,
/// }
///
/// map_entity!(RegisterUserResponse {
///     from user: User => {
///         id: user.id,
///         email: user.email,
///     }
/// });
///
/// // usage:
/// let u = User { id: 1, email: "ada@example.com".into() };
/// let resp = RegisterUserResponse::from_source(u);
/// ```
///
/// ## Version 3: `Target::from_parts(a, b, ...)`
///
/// Syntax:
///
/// ```rust,ignore
/// map_entity!(TargetType {
///     from a: AType, b: BType, /* ... */ => {
///         field_a: /* expr using `a`, `b`, ... */,
///         field_b: /* expr using `a`, `b`, ... */,
///     }
/// });
/// ```
///
/// This generates:
///
/// ```rust,ignore
/// impl TargetType {
///     pub fn from_parts(a: AType, b: BType, ...) -> Self {
///         Self { field_a: ..., field_b: ... }
///     }
/// }
/// ```
///
/// ### Example
///
/// ```rust,ignore
/// #[derive(Clone)]
/// struct RegisterUserRequest {
///     email: String,
/// }
///
/// #[derive(Clone)]
/// struct User {
///     id: u32,
/// }
///
/// struct Identity {
///     user_id: u32,
///     email: String,
/// }
///
/// map_entity!(Identity {
///     from req: RegisterUserRequest, user: User => {
///         user_id: user.id,
///         email: req.email,
///     }
/// });
///
/// // usage:
/// let req = RegisterUserRequest { email: "ada@example.com".into() };
/// let user = User { id: 42 };
/// let identity = Identity::from_parts(req, user);
/// ```
///
/// ## Common pitfalls
///
/// - **Moving vs borrowing:** Version 1 consumes the `source` value. If you want to map from a
///   reference (`&Source`), define a separate mapping (e.g., `&Source => Target`) or use version 2/3
///   with a borrowed type (`from src: &Source => { ... }`).
/// - **Field ownership:** If a field is a `String` and you use `source.email`, it moves the string.
///   If you need to keep the source value, use `source.email.clone()` (or pass by reference).
/// - **Conflicting method names:** Versions 2 and 3 generate `from_source` / `from_parts` methods.
///   Don’t use them on the same target type more than once unless you intend to overwrite or cause
///   duplicate-definition compiler errors.
#[macro_export]
macro_rules! map_entity {
    // Version 1: Simple mapping with From trait
    (
        $from:ty => $to:ty {
            $($field:ident: $expr:expr),* $(,)?
        }
    ) => {
        impl From<$from> for $to {
            fn from(source: $from) -> Self {
                Self {
                    $(
                        $field: $expr,
                    )*
                }
            }
        }
    };

    // Version 2: Custom function with explicit source
    (
        $to:ty {
            from $source:ident: $source_type:ty
            => { $($field:ident: $expr:expr),* $(,)? }
        }
    ) => {
        impl $to {
            pub fn from_source($source: $source_type) -> Self {
                Self {
                    $(
                        $field: $expr,
                    )*
                }
            }
        }
    };

    // Version 3: Multiple sources
    (
        $to:ty {
            from $($arg:ident: $arg_type:ty),*
            => { $($field:ident: $expr:expr),* $(,)? }
        }
    ) => {
        impl $to {
            pub fn from_parts($($arg: $arg_type),*) -> Self {
                Self {
                    $(
                        $field: $expr,
                    )*
                }
            }
        }
    };
}
