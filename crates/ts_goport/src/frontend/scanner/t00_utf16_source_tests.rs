//! Regression for retained JavaScript UTF-16 source, not malformed UTF-8 bytes.
use super::*;
use std::{cell::RefCell, rc::Rc};

fn first_tokens(text: &str) -> (SyntaxKind, SyntaxKind, Vec<u32>) {
    let errors = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&errors);
    let mut scanner = new_scanner();
    scanner.set_skip_trivia(true);
    scanner.set_text(text);
    scanner.set_on_error(Some(Box::new(move |message, _, _, _| {
        observed.borrow_mut().push(message.code());
    })));
    let first = scanner.scan();
    let second = scanner.scan();
    let codes = errors.borrow().clone();
    (first, second, codes)
}

#[test]
fn utf16_lone_surrogates_are_invalid_characters_and_scanning_continues() {
    for bytes in [[0xed, 0xa0, 0x81], [0xed, 0xb0, 0x82]] {
        let mut input = bytes.to_vec();
        input.extend_from_slice(b"; export const independentContinuation = 73;");
        let source = go_string_from_bytes(input.clone());
        let source = fuse_surrogate_bytes(&source).into_owned();
        assert_eq!(&*go_string_bytes(&source), input.as_slice());
        assert!(matches!(go_unit_at(&source, 0).0, GoUnit::Surrogate(_)));
        let (first, second, codes) = first_tokens(&source);
        assert_eq!(first, SyntaxKind::Unknown);
        assert_eq!(second, SyntaxKind::SemicolonToken);
        assert_eq!(codes, vec![1127]);
    }
}

#[test]
fn literal_replacement_character_and_invalid_bytes_remain_binary() {
    for source in [
        go_string_from_utf8("\u{fffd}; export {};".into()),
        go_string_from_bytes(vec![0xff, b';']),
    ] {
        let (first, second, codes) = first_tokens(&source);
        assert_eq!(first, SyntaxKind::NonTextFileMarkerTrivia);
        assert_eq!(second, SyntaxKind::EndOfFile);
        assert_eq!(codes, vec![1490]);
    }
}
