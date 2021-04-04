use rusty_v8 as v8;

static BUNDLE_PATH: &str = "./bundle/dist/main.js";
static RENDERER_PATH: &str = "./renderer/dist/main.js";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

trait Catch {
    type R;
    fn catch(self, tc: &mut v8::TryCatch<v8::HandleScope>) -> Self::R;
}

impl<T> Catch for Option<T> {
    type R = Result<T>;
    fn catch(self, tc: &mut v8::TryCatch<v8::HandleScope>) -> Self::R {
        self.ok_or_else(|| {
            match tc.message() {
                Some(msg) => msg.get(tc).to_rust_string_lossy(tc),
                None => "No exception message available".to_string(),
            }
            .into()
        })
    }
}

fn bundle_import_callback<'a>(
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let tc = &mut v8::TryCatch::new(scope);

    let error_handler = |e| panic!("Error in callback: {}", e);

    let specifier = specifier
        .to_string(tc)
        .catch(tc)
        .map_err(error_handler)
        .ok()?
        .to_rust_string_lossy(tc);

    assert_eq!(specifier, "__SSR_BUNDLE__");

    let bundle_src_string = v8::String::new_from_utf8(
        tc,
        std::fs::read(BUNDLE_PATH).ok()?.as_slice(),
        v8::NewStringType::Normal,
    )
    .catch(tc)
    .map_err(error_handler)
    .ok()?;

    let empty = v8::String::empty(tc);

    let bundle_src = v8::script_compiler::Source::new(
        bundle_src_string,
        Some(&v8::ScriptOrigin::new(
            tc,
            empty.into(),
            0,
            0,
            false,
            0,
            empty.into(),
            false,
            false,
            true,
        )),
    );

    let bundle_module = v8::script_compiler::compile_module(tc, bundle_src)
        .catch(tc)
        .map_err(error_handler)
        .ok()?;

    Some(bundle_module)
}

fn main() -> Result<()> {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let isolate_scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(isolate_scope);
    let scope = &mut v8::ContextScope::new(isolate_scope, context);
    let tc = &mut v8::TryCatch::new(scope);

    let empty = v8::String::empty(tc);

    let renderer_src_string = v8::String::new_from_utf8(
        tc,
        std::fs::read(RENDERER_PATH)?.as_slice(),
        v8::NewStringType::Normal,
    )
    .catch(tc)?;

    let renderer_src = v8::script_compiler::Source::new(
        renderer_src_string,
        Some(&v8::ScriptOrigin::new(
            tc,
            empty.into(),
            0,
            0,
            false,
            0,
            empty.into(),
            false,
            false,
            true,
        )),
    );

    let renderer_module = v8::script_compiler::compile_module(tc, renderer_src).catch(tc)?;

    renderer_module
        .instantiate_module(tc, bundle_import_callback)
        .catch(tc)?;

    let renderer_namespace = renderer_module
        .get_module_namespace()
        .to_object(tc)
        .catch(tc)?;

    renderer_module.evaluate(tc).catch(tc)?;

    let default = v8::String::new(tc, "default").catch(tc)?;
    let default_value = renderer_namespace.get(tc, default.into()).catch(tc)?;
    let render_fn = unsafe { v8::Local::<v8::Function>::cast(default_value) };

    let mut args = std::env::args();
    args.next().ok_or("Must be called with at least one arg")?;
    let props = args.next().ok_or("Must be called with at least one arg")?;
    let props_str = v8::String::new(tc, &props).catch(tc)?;
    let parsed_props = v8::json::parse(tc, props_str).catch(tc)?;

    let empty_object = v8::Object::new(tc);
    let rendered = render_fn
        .call(tc, empty_object.into(), &[parsed_props])
        .catch(tc)?
        .to_string(tc)
        .catch(tc)?
        .to_rust_string_lossy(tc);

    println!("{}", rendered);

    Ok(())
}
