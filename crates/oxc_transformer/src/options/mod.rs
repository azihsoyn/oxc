pub mod babel;
mod env;

use std::path::PathBuf;

use oxc_diagnostics::Error;

pub use env::EnvOptions;

use crate::{
    common::helper_loader::{HelperLoaderMode, HelperLoaderOptions},
    compiler_assumptions::CompilerAssumptions,
    es2015::ES2015Options,
    es2016::ES2016Options,
    es2017::ES2017Options,
    es2018::ES2018Options,
    es2019::ES2019Options,
    es2020::ES2020Options,
    es2021::ES2021Options,
    es2022::ES2022Options,
    jsx::JsxOptions,
    regexp::RegExpOptions,
    typescript::TypeScriptOptions,
    ReactRefreshOptions,
};

use babel::BabelOptions;

/// <https://babel.dev/docs/options>
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    //
    // Primary Options
    //
    /// The working directory that all paths in the programmatic options will be resolved relative to.
    pub cwd: PathBuf,

    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// Jsx Transform
    ///
    /// See [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub jsx: JsxOptions,

    /// ECMAScript Env Options
    pub env: EnvOptions,

    pub helper_loader: HelperLoaderOptions,
}

impl TransformOptions {
    /// Explicitly enable all plugins that are ready, mainly for testing purposes.
    pub fn enable_all() -> Self {
        Self {
            cwd: PathBuf::new(),
            assumptions: CompilerAssumptions::default(),
            typescript: TypeScriptOptions::default(),
            jsx: JsxOptions {
                development: true,
                refresh: Some(ReactRefreshOptions::default()),
                ..JsxOptions::default()
            },
            env: EnvOptions::enable_all(),
            helper_loader: HelperLoaderOptions {
                mode: HelperLoaderMode::Runtime,
                ..Default::default()
            },
        }
    }
}

impl TryFrom<&BabelOptions> for TransformOptions {
    type Error = Vec<Error>;

    /// If the `options` contains any unknown fields, they will be returned as a list of errors.
    fn try_from(options: &BabelOptions) -> Result<Self, Self::Error> {
        let mut errors = Vec::<Error>::new();
        errors.extend(options.plugins.errors.iter().map(|err| Error::msg(err.clone())));
        errors.extend(options.presets.errors.iter().map(|err| Error::msg(err.clone())));

        let typescript = options
            .presets
            .typescript
            .clone()
            .or_else(|| options.plugins.typescript.clone())
            .unwrap_or_default();

        let jsx = if let Some(options) = &options.presets.jsx {
            options.clone()
        } else {
            let mut jsx_options = if let Some(options) = &options.plugins.react_jsx_dev {
                options.clone()
            } else if let Some(options) = &options.plugins.react_jsx {
                options.clone()
            } else {
                JsxOptions::default()
            };
            jsx_options.development = options.plugins.react_jsx_dev.is_some();
            jsx_options.jsx_plugin = options.plugins.react_jsx.is_some();
            jsx_options.display_name_plugin = options.plugins.react_display_name;
            jsx_options.jsx_self_plugin = options.plugins.react_jsx_self;
            jsx_options.jsx_source_plugin = options.plugins.react_jsx_source;
            jsx_options
        };

        let env = options.presets.env.clone().unwrap_or_default();

        let regexp = RegExpOptions {
            sticky_flag: env.regexp.sticky_flag || options.plugins.sticky_flag,
            unicode_flag: env.regexp.unicode_flag || options.plugins.unicode_flag,
            dot_all_flag: env.regexp.dot_all_flag || options.plugins.dot_all_flag,
            look_behind_assertions: env.regexp.look_behind_assertions
                || options.plugins.look_behind_assertions,
            named_capture_groups: env.regexp.named_capture_groups
                || options.plugins.named_capture_groups,
            unicode_property_escapes: env.regexp.unicode_property_escapes
                || options.plugins.unicode_property_escapes,
            match_indices: env.regexp.match_indices,
            set_notation: env.regexp.set_notation || options.plugins.set_notation,
        };

        let es2015 = ES2015Options {
            arrow_function: options.plugins.arrow_function.or(env.es2015.arrow_function),
        };

        let es2016 = ES2016Options {
            exponentiation_operator: options.plugins.exponentiation_operator
                || env.es2016.exponentiation_operator,
        };

        let es2017 = ES2017Options {
            async_to_generator: options.plugins.async_to_generator || env.es2017.async_to_generator,
        };

        let es2018 = ES2018Options {
            object_rest_spread: options
                .plugins
                .object_rest_spread
                .or(env.es2018.object_rest_spread),
            async_generator_functions: options.plugins.async_generator_functions
                || env.es2018.async_generator_functions,
        };

        let es2019 = ES2019Options {
            optional_catch_binding: options.plugins.optional_catch_binding
                || env.es2019.optional_catch_binding,
        };

        let es2020 = ES2020Options {
            nullish_coalescing_operator: options.plugins.nullish_coalescing_operator
                || env.es2020.nullish_coalescing_operator,
        };

        let es2021 = ES2021Options {
            logical_assignment_operators: options.plugins.logical_assignment_operators
                || env.es2021.logical_assignment_operators,
        };

        let es2022 = ES2022Options {
            class_static_block: options.plugins.class_static_block || env.es2022.class_static_block,
            class_properties: options.plugins.class_properties.or(env.es2022.class_properties),
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        let helper_loader = HelperLoaderOptions {
            mode: if options.external_helpers {
                HelperLoaderMode::External
            } else {
                HelperLoaderMode::default()
            },
            ..HelperLoaderOptions::default()
        };

        Ok(Self {
            cwd: options.cwd.clone().unwrap_or_default(),
            assumptions: options.assumptions,
            typescript,
            jsx,
            env: EnvOptions {
                regexp,
                es2015,
                es2016,
                es2017,
                es2018,
                es2019,
                es2020,
                es2021,
                es2022,
            },
            helper_loader,
        })
    }
}
