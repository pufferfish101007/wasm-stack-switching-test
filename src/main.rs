use std::{borrow::Cow, io::Write};
use wasm_encoder::{
    BlockType, ContType, ElementSection, Elements, EntityType, Handle, HeapType, ImportSection, RefType, StartSection, SubType, TagKind, TagSection, TagType
};

fn main() {
    use wasm_encoder::{
        CodeSection, ExportKind, ExportSection, Function, FunctionSection, Module, TypeSection,
        ValType,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![]);
    let empty_func_ty = 0;
    types.ty().cont(&ContType(empty_func_ty));
    let cont_ty = 1;
    types.ty().function(vec![ValType::I32], vec![]);
    types.ty().function(
        vec![],
        vec![
            ValType::I32,
            ValType::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(cont_ty),
            }),
        ],
    );
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("tests", "log", EntityType::Function(2));
    module.section(&imports);

    let mut functions = FunctionSection::new();
    functions.function(empty_func_ty);
    functions.function(empty_func_ty);
    module.section(&functions);

    let mut tags = TagSection::new();
    tags.tag(TagType {
        kind: TagKind::Exception,
        func_type_idx: 2,
    });
    module.section(&tags);

    let mut exports = ExportSection::new();
    exports.export("consumer", ExportKind::Func, 2);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.declared(Elements::Functions(Cow::Borrowed(&[1])));
    module.section(&elements);

    let mut codes = CodeSection::new();

    let generator_locals = vec![ValType::I32];
    let mut generator = Function::new_with_locals_types(generator_locals);
    generator
        .instructions()
        .i32_const(10)
        .local_set(0)
        .loop_(BlockType::Empty)
        .local_get(0)
        .suspend(0)
        .local_get(0)
        .i32_const(1)
        .i32_sub()
        .local_tee(0)
        .br_if(0)
        .end()
        .end();
    codes.function(&generator);

    let consumer_locals = vec![ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::Concrete(cont_ty),
    })];
    let mut consumer = Function::new_with_locals_types(consumer_locals);
    consumer
        .instructions()
        .ref_func(1)
        .cont_new(cont_ty)
        .local_set(0)
        .loop_(BlockType::Empty)
        .block(BlockType::FunctionType(3))
        .local_get(0)
        .resume(cont_ty, [Handle::OnLabel { tag: 0, label: 0 }])
        .return_()
        .end()
        .local_set(0)
        .call(0)
        .br(0)
        .end()
        .end();
    codes.function(&consumer);

    module.section(&codes);

    // Extract the encoded Wasm bytes for this module.
    let wasm_bytes = module.finish();

    eprintln!("wasm module size (bytes): {}", wasm_bytes.len());

    if let Err(e) = wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&wasm_bytes)
    {
        eprintln!("{e:?}");
    }
    std::io::stdout()
        .write(&wasm_bytes)
        .expect("write to stdout failed");
}
