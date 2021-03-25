fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}
