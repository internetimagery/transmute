use cpython::{py_class, py_module_initializer, ObjectProtocol, PyObject, PyResult, PythonObject};
use search::{Graph, Int};
use std::cell::RefCell;
mod search;

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
        cost: Int,
        type_in: &PyObject,
        type_out: &PyObject
    ) -> PyResult<PyObject> {
        let hash_in = type_in.hash(py)?;
        let hash_out = type_out.hash(py)?;
        self.graph(py).borrow_mut().add_edge(cost, hash_in, hash_out);
        Ok(py.None())
    }
    def transmute(
        &self,
        value: &PyObject,
        type_out: &PyObject,
        type_in: Option<&PyObject> = None
    ) -> PyResult<PyObject> {
        let hash_in = match type_in {
            Some(type_override) => type_override.hash(py)?,
            None => value.get_type(py).into_object().hash(py)?
        };
        let hash_out = type_out.hash(py)?;
        self.graph(py).borrow().search(hash_in, hash_out);
        Ok(py.None())
    }
});
