use cpython::{py_class, py_module_initializer, ObjectProtocol, PyObject, PyResult, PythonObject};
use std::cell::RefCell;
mod search;
use search::Graph;

// Note transmute is name of library in Cargo.toml
py_module_initializer!(transmute, |py, m| {
    m.add(py, "__doc__", "Transmutation!")?;
    m.add_class::<Grimoire>(py)?;
    Ok(())
});

py_class!(class Grimoire |py| {
    data graph: RefCell<Graph>;
    def __new__(_cls) -> PyResult<Grimoire> {
        Grimoire::create_instance(py, RefCell::new(Graph::new()))
    }
    def inscribe_transmutation(
        &self,
        cost: isize,
        type_in: PyObject,
        type_out: PyObject
    ) -> PyResult<bool> {
        let hash_in = type_in.hash(py)?;
        let hash_out = type_out.hash(py)?;
        self.graph(py).borrow_mut().add(cost, hash_in, hash_out);
        Ok(true)
    }
    def transmute(
        &self,
        value: PyObject,
        type_out: PyObject
    ) -> PyResult<bool> {
        let hash_in = value.get_type(py).into_object().hash(py)?;
        let hash_out = type_out.hash(py)?;
        self.graph(py).borrow().search(hash_in, hash_out);
        Ok(true)
    }
});
