use std::collections::VecDeque;
use syn::UseTree;

fn recurse_use_item<'a>(use_tree: &'a UseTree, tokens: &mut VecDeque<&str>) -> Option<&'a UseTree> {
    let curr_ident = tokens.pop_front().unwrap_or_default();
    match use_tree {
        UseTree::Path(use_stmt_third) => {
            if use_stmt_third.ident == curr_ident {
                recurse_use_item(&*use_stmt_third.tree, tokens)
            } else {
                None
            }
        }
        UseTree::Name(ref use_stmt_third) => {
            if use_stmt_third.ident == curr_ident {
                Some(use_tree)
            } else {
                None
            }
        }
        UseTree::Rename(_) => None,
        UseTree::Glob(_) => None,
        UseTree::Group(ref use_stmt_third) => {
            for item in &use_stmt_third.items {
                tokens.push_front(curr_ident);
                if let Some(group_tree) = recurse_use_item(item, tokens) {
                    return Some(group_tree);
                }
            }
            None
        }
    }
}

pub fn traverse_use_item<'a>(use_tree: &'a UseTree, tokens: Vec<&str>) -> Option<&'a UseTree> {
    let mut tokens = VecDeque::from(tokens);
    recurse_use_item(use_tree, &mut tokens)
}
