use cpython::{
    py_class, py_module_initializer, ObjectProtocol, PyDrop, PyObject, PyResult, PythonObject,
};
use search::{Graph, Int};
use std::cell::RefCell;
use std::collections::HashMap;
mod search;

// Note transmute is name of library in Cargo.toml
py_module_initializer!(transmute, |py, m| {
    m.add(py, "__doc__", "Transmutation!")?;
    m.add_class::<Grimoire>(py)?;
    Ok(())
});

py_class!(class Grimoire |py| {
    data graph: RefCell<Graph>;
    data functions: RefCell<HashMap<Int, PyObject>>;
    def __new__(_cls) -> PyResult<Grimoire> {
        Grimoire::create_instance(
            py,
            RefCell::new(Graph::new()),
            RefCell::new(HashMap::new()),
        )
    }
    def inscribe_transmutation(
        &self,
        cost: Int,
        type_in: &PyObject,
        type_out: &PyObject,
        function: PyObject
    ) -> PyResult<PyObject> {
        let hash_in = type_in.hash(py)?;
        let hash_out = type_out.hash(py)?;
        let hash_func = function.hash(py)?;
        // Store a reference to the python object in this outer layer
        // but refer to it via its hash.
        self.functions(py).borrow_mut().insert(hash_func, function);
        self.graph(py).borrow_mut().add_edge(cost, hash_in, hash_out, hash_func);
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

    ///////////////////////////////////////////////////////////////
    // Satisfy python garbage collector
    // because we hold a reference to the functions provided
    def __traverse__(&self, visit) {
        for function in self.functions(py).borrow().values() {
            visit.call(function)?
        }
        Ok(())
    }

    def __clear__(&self) {
        for (_, func) in self.functions(py).borrow_mut().drain() {
            func.release_ref(py);
        }
    }
    ///////////////////////////////////////////////////////////////
});
