use syn::{Token, AngleBracketedGenericArguments, GenericArgument, Ident, ParenthesizedGenericArguments, Path, PathArguments, PathSegment, QSelf, Type, TypePath, punctuated::Punctuated};
use std::collections::HashSet;

pub fn transform_type_path_in_place(type_path: &mut TypePath, type_ident: Ident, (trait_ident, trait_args): (&Ident, &PathArguments), skip: &HashSet<String>) {
    // First, recursively process any nested types in the path arguments
    transform_path_arguments_recursively(&mut type_path.path.segments, type_ident.clone(), (trait_ident, trait_args), skip);
    
    // Check if the path contains "Self" as the first segment
    let has_self = type_path.path.segments
        .first()
        .map(|segment| segment.ident == "Self")
        .unwrap_or(false);
    
    // Only proceed with the main transformation if the path starts with "Self"
    if !has_self {
        return;
    }

    // Check if the identifier following "Self" should be skipped
    if let Some(second_segment) = type_path.path.segments.iter().nth(1) {
        if skip.contains(&second_segment.ident.to_string()) {
            return;
        }
    }

    // Create the Self::U type for the QSelf
    let self_u_type = Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: vec![
                PathSegment {
                    ident: Ident::new("Self", proc_macro2::Span::call_site()),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: type_ident,
                    arguments: PathArguments::None,
                }
            ].into_iter().collect(),
        }
    });

    // Create the new QSelf for <Self::U as Z>
    let qself = QSelf {
        ty: Box::new(self_u_type),
        position: 1, // Position after the trait name (Z) in the path
        as_token: Some(syn::token::As::default()),
        gt_token: syn::token::Gt::default(),
        lt_token: syn::token::Lt::default(),
    };

    // Collect the remaining segments (skip the first "Self" segment)
    let remaining_segments: Vec<PathSegment> = type_path.path.segments
        .iter()
        .skip(1)
        .cloned()
        .collect();

    let trait_args = match trait_args {
        PathArguments::AngleBracketed(generics) => {
            let mut angle_bracketed = generics.clone();
            angle_bracketed.args = generics.args.iter().filter_map(|arg|{
                match arg {
                    GenericArgument::Const(_) => Some(arg.clone()),
                    GenericArgument::Lifetime(_) => Some(arg.clone()),
                    GenericArgument::Type(_) => Some(arg.clone()),
                    _ => None,
                }
            }).collect::<Punctuated<GenericArgument, Token![,]>>();
            PathArguments::AngleBracketed(angle_bracketed)
        },
        PathArguments::Parenthesized(_) => PathArguments::None,
        PathArguments::None => PathArguments::None
    };

    // Create new segments starting with the trait name
    let mut new_segments = vec![
        PathSegment {
            ident: trait_ident.clone(),
            arguments: trait_args,
        }
    ];
    new_segments.extend(remaining_segments);

    // Update the TypePath in place
    type_path.qself = Some(qself);
    type_path.path.segments = new_segments.into_iter().collect();
    type_path.path.leading_colon = None;
}

/// Recursively transforms path arguments to handle nested Self references
fn transform_path_arguments_recursively(
    segments: &mut syn::punctuated::Punctuated<PathSegment, syn::token::PathSep>,
    type_ident: Ident,
    (trait_ident, trait_args): (&Ident, &PathArguments),
    skip: &HashSet<String>
) {
    for segment in segments.iter_mut() {
        match &mut segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed) => {
                transform_angle_bracketed_args(angle_bracketed, type_ident.clone(), (trait_ident, trait_args), skip);
            }
            PathArguments::Parenthesized(parenthesized) => {
                transform_parenthesized_args(parenthesized, type_ident.clone(), (trait_ident, trait_args), skip);
            }
            PathArguments::None => {
                // No arguments to process
            }
        }
    }
}

/// Transforms angle-bracketed generic arguments (e.g., Vec<Self>)
fn transform_angle_bracketed_args(
    args: &mut AngleBracketedGenericArguments,
    type_ident: Ident,
    (trait_ident, trait_args): (&Ident, &PathArguments),
    skip: &HashSet<String>
) {
    for arg in args.args.iter_mut() {
        match arg {
            GenericArgument::Type(ty) => {
                transform_type_recursively(ty, type_ident.clone(), (trait_ident, trait_args), skip);
            }
            GenericArgument::AssocType(assoc_type) => {
                transform_type_recursively(&mut assoc_type.ty, type_ident.clone(), (trait_ident, trait_args), skip);
            }
            // Other generic argument types (Const, AssocConst, Constraint) don't contain types
            _ => {}
        }
    }
}

/// Transforms parenthesized arguments (e.g., function types)
fn transform_parenthesized_args(
    args: &mut ParenthesizedGenericArguments,
    type_ident: Ident,
    (trait_ident, trait_args): (&Ident, &PathArguments),
    skip: &HashSet<String>
) {
    // Transform input types
    for input in args.inputs.iter_mut() {
        transform_type_recursively(input, type_ident.clone(), (trait_ident, trait_args), skip);
    }
    
    // Transform output type if present
    if let syn::ReturnType::Type(_, output_type) = &mut args.output {
        transform_type_recursively(output_type, type_ident.clone(), (trait_ident, trait_args), skip);
    }
}

/// Recursively transforms any Type that might contain Self references
fn transform_type_recursively(ty: &mut Type, type_ident: Ident, (trait_ident, trait_args): (&Ident, &PathArguments), skip: &HashSet<String>) {
    match ty {
        Type::Path(type_path) => {
            // Recursively transform this TypePath
            transform_type_path_in_place(type_path, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Reference(type_ref) => {
            // Transform the referenced type
            transform_type_recursively(&mut type_ref.elem, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Ptr(type_ptr) => {
            // Transform the pointed-to type
            transform_type_recursively(&mut type_ptr.elem, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Slice(type_slice) => {
            // Transform the slice element type
            transform_type_recursively(&mut type_slice.elem, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Array(type_array) => {
            // Transform the array element type
            transform_type_recursively(&mut type_array.elem, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Tuple(type_tuple) => {
            // Transform all tuple element types
            for elem in type_tuple.elems.iter_mut() {
                transform_type_recursively(elem, type_ident.clone(), (trait_ident, trait_args), skip);
            }
        }
        Type::Paren(type_paren) => {
            // Transform the parenthesized type
            transform_type_recursively(&mut type_paren.elem, type_ident, (trait_ident, trait_args), skip);
        }
        Type::Group(type_group) => {
            // Transform the grouped type
            transform_type_recursively(&mut type_group.elem, type_ident, (trait_ident, trait_args), skip);
        }
        // Other type variants (ImplTrait, TraitObject, etc.) either don't contain
        // nested types or are more complex and would need specific handling
        _ => {}
    }
}