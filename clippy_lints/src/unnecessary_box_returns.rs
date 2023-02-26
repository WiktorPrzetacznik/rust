use clippy_utils::diagnostics::span_lint_and_then;
use rustc_errors::Applicability;
use rustc_hir::{def_id::LocalDefId, FnDecl, FnRetTy, ImplItemKind, Item, ItemKind, Node, TraitItem, TraitItemKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};

declare_clippy_lint! {
    /// ### What it does
    ///
    /// Checks for a return type containing a `Box<T>` where `T` implements `Sized`
    ///
    /// ### Why is this bad?
    ///
    /// It's better to just return `T` in these cases. The caller may not need
    /// the value to be boxed, and it's expensive to free the memory once the
    /// `Box<T>` been dropped.
    ///
    /// ### Example
    /// ```rust
    /// fn foo() -> Box<String> {
    ///     Box::new(String::from("Hello, world!"))
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// fn foo() -> String {
    ///     String::from("Hello, world!")
    /// }
    /// ```
    #[clippy::version = "1.70.0"]
    pub UNNECESSARY_BOX_RETURNS,
    pedantic,
    "Needlessly returning a Box"
}

pub struct UnnecessaryBoxReturns {
    avoid_breaking_exported_api: bool,
}

impl_lint_pass!(UnnecessaryBoxReturns => [UNNECESSARY_BOX_RETURNS]);

impl UnnecessaryBoxReturns {
    pub fn new(avoid_breaking_exported_api: bool) -> Self {
        Self {
            avoid_breaking_exported_api,
        }
    }

    fn check_fn_decl(&mut self, cx: &LateContext<'_>, decl: &FnDecl<'_>, def_id: LocalDefId) {
        // we don't want to tell someone to break an exported function if they ask us not to
        if self.avoid_breaking_exported_api && cx.effective_visibilities.is_exported(def_id) {
            return;
        }

        let FnRetTy::Return(return_ty_hir) = &decl.output else { return };

        let return_ty = cx
            .tcx
            .erase_late_bound_regions(cx.tcx.fn_sig(def_id).skip_binder())
            .output();

        if !return_ty.is_box() {
            return;
        }

        let boxed_ty = return_ty.boxed_ty();

        // it's sometimes useful to return Box<T> if T is unsized, so don't lint those
        if boxed_ty.is_sized(cx.tcx, cx.param_env) {
            span_lint_and_then(
                cx,
                UNNECESSARY_BOX_RETURNS,
                return_ty_hir.span,
                format!("boxed return of the sized type `{boxed_ty}`").as_str(),
                |diagnostic| {
                    diagnostic.span_suggestion(
                        return_ty_hir.span,
                        "try",
                        boxed_ty.to_string(),
                        // the return value and function callers also needs to
                        // be changed, so this can't be MachineApplicable
                        Applicability::Unspecified,
                    );
                    diagnostic.help("changing this also requires a change to the return expressions in this function");
                },
            );
        }
    }
}

impl LateLintPass<'_> for UnnecessaryBoxReturns {
    fn check_trait_item(&mut self, cx: &LateContext<'_>, item: &TraitItem<'_>) {
        let TraitItemKind::Fn(signature, _) = &item.kind else { return };
        self.check_fn_decl(cx, signature.decl, item.owner_id.def_id);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'_>, item: &rustc_hir::ImplItem<'_>) {
        // Ignore implementations of traits, because the lint should be on the
        // trait, not on the implmentation of it.
        let Node::Item(parent) = cx.tcx.hir().get_parent(item.hir_id()) else { return };
        let ItemKind::Impl(parent) = parent.kind else { return };
        if parent.of_trait.is_some() {
            return;
        }

        let ImplItemKind::Fn(signature, ..) = &item.kind else { return };
        self.check_fn_decl(cx, signature.decl, item.owner_id.def_id);
    }

    fn check_item(&mut self, cx: &LateContext<'_>, item: &Item<'_>) {
        let ItemKind::Fn(signature, ..) = &item.kind else { return };
        self.check_fn_decl(cx, signature.decl, item.owner_id.def_id);
    }
}
