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

    /// Write a function into the grimoire so it may be used as a piece in the transmutation chain later.
    /// Eventually a transmutation chain will consist of a number of these placed back to back.
    /// So the simpler and smaller the transmutation the better.

    /// Args:
    ///     cost:
    ///         A number representing how much work this transmuter needs to do.
    ///         Lower numbers are prioritized.
    ///         eg: just getting an attribute would be a low number. Accessing an http service would be higher etc
    ///     type_in:
    ///         Type of input expected.
    ///         Typically a class type, but can be anything hashable that is relevant to a workflow.
    ///         eg str / MyClass or a composite type eg frozenset([Type1, Type2])
    ///     variations_in:
    ///         A sequence of hashable "tags" further describing the input type.
    ///         For the node to be used, all these variations are required (dependencies).
    ///         This is useful if the more simple type is not enough by itself.
    ///         eg: str (can be path/href/name/any concept)
    ///     type_out:
    ///         Same as "type_in", but representing the output of the transmutation.
    ///         NOTE: it is important the transmuter only outputs the stated type (eg no None option)
    ///     variations_out:
    ///         Same as "variations_in" except that variations are descriptive and not dependencies.
    ///         They can satisfy dependencies for transmuters further down the chain.
    ///     function:
    ///         The transmuter itself. Take a single input, produce a single output.
    ///         It is important that only an simple transmutation is made, and that any deviation is raised as an Error.
    ///         eg: maybe some attribute is not available and usually you'd return None. There is no strict type
    ///         checking here, so raise an error and bail instead.
    def inscribe_transmutation(
        &self,
        cost: Int,
        type_in: &PyObject,
        variations_in: &PyObject,
        type_out: &PyObject,
        variations_out: &PyObject,
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

    /// From a given type, attempt to produce a requested type.
    /// OR from some given data, attempt to traverse links to get the requested data.

    /// Args:
    ///     value: The input you have going into the process. This can be anything.
    ///     type_want:
    ///         The type you want to recieve. A chain of transmuters will be produced
    ///         attempting to attain this type.
    ///     variations_want:
    ///         A sequence of variations further describing the type you wish to attain.
    ///         This is optional but can help guide a transmutation through more complex types.
    ///     type_have:
    ///         An optional override for the starting type.
    ///         If not provided the type of the value is taken instead.
    ///     variations_have:
    ///         Optionally include any extra variations to the input.
    ///         If context is known but hard to detect this can help direct a more complex
    ///         transmutation.
    ///     explicit:
    ///         If this is True, the variations_have attribute will entirely override
    ///         any detected tags. Use this if the detection is bad and you know EXACTLY what you need.
    /// Returns:
    ///     Any: Whatever the result requested happens to be
    def transmute(
        &self,
        value: PyObject,
        type_out: &PyObject,
        variations_out: Option<&PyObject> = None,
        type_in: Option<&PyObject> = None,
        variations_in: Option<&PyObject> = None,
        explicit: Option<bool> = None
    ) -> PyResult<PyObject> {
        let hash_in = match type_in {
            Some(type_override) => type_override.hash(py)?,
            None => value.get_type(py).into_object().hash(py)?
        };
        let hash_out = type_out.hash(py)?;
        if let Some(edges) = self.graph(py).borrow().search(hash_in, hash_out) {
            let functions = self.functions(py).borrow();
            let mut result = value;
            for edge in edges {
               let func = functions.get(&edge.hash_func).expect("Function should exist");
               result = func.call(py, (result,), None).expect("Darn an error... need that handling");
            }
            return Ok(result)
        }
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
