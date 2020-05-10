use cpython::{
    py_class, py_exception, py_module_initializer, ObjectProtocol, PyClone, PyDrop, PyErr,
    PyIterator, PyObject, PyResult, PySequence, PythonObject,
};
use search::{Graph, Int};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
mod search;

// Simple utility, make a hash set out of python sequence
macro_rules! hash_seq {
    ($py:expr, $seq:expr) => {
        $seq.iter($py)?
            .filter_map(|v| match v {
                Ok(v) => v.hash($py).ok(),
                _ => None,
            })
            .collect()
    };
}

//////////////////////////////////////////////////
// MODULE SETUP
// Note transmute is name of library in Cargo.toml
py_module_initializer!(transmute, |py, m| {
    m.add(py, "__doc__", "Simple network housing a collection of equally simple \"a to b\" functions.
        Providing the ability to chain a bunch of them together for more complex transmutations.

        You could be wanting to transmute between a chain of types, or traverse a bunch of object oriented links.
        If you're often thinking \"I have this, how can I get that\", then this type of solution could help.

        >>> lab = Lab()
        >>> lab.stock_reagent(1, str, [\"href\"], WebPage, [], load_webpage)
        >>> lab.inscribe_detector(str, http_detector)
        >>> lab.transmute(\"http://somewhere.html\", WebPage)
    ")?;
    m.add(py, "TransmuteFailure", py.get_type::<TransmuteFailure>())?;
    m.add(
        py,
        "LackingReagentFailure",
        py.get_type::<LackingReagentFailure>(),
    )?;
    m.add(py, "CommandFailure", py.get_type::<CommandFailure>())?;
    m.add_class::<Lab>(py)?;
    Ok(())
});
//////////////////////////////////////////////////

//////////////////////////////////////////////////
// Exceptions
py_exception!(transmute, TransmuteFailure); // Root exception
py_exception!(transmute, LackingReagentFailure, TransmuteFailure);
py_exception!(transmute, CommandFailure, TransmuteFailure);
//////////////////////////////////////////////////

py_class!(class Lab |py| {
    data graph: RefCell<Graph>;
    data functions: RefCell<HashMap<Int, PyObject>>;
    data activators: RefCell<HashMap<Int, Vec<PyObject>>>;
    def __new__(_cls) -> PyResult<Lab> {
        Lab::create_instance(
            py,
            RefCell::new(Graph::new()),
            RefCell::new(HashMap::new()),
            RefCell::new(HashMap::new()),
        )
    }

    /// Add a function so it may be used as a step in the transmutation process later.
    /// Eventually a transmutation chain will consist of a number of these placed back to back.
    /// So the simpler, smaller and more focused the function the better.
    ///
    /// Args:
    ///     cost:
    ///         A number representing how much work this function needs to do.
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
    ///         NOTE: it is important the function only outputs the stated type (eg error if
    ///         otherwise it'd return None)
    ///     variations_out:
    ///         Same as "variations_in" except that variations are descriptive and not dependencies.
    ///         They can satisfy dependencies for transmuters further down the chain.
    ///     function:
    ///         The reagent itself. Take a single input, produce a single output.
    ///         It is important that only an simple transmutation is made, and that any deviation is raised as an Error.
    ///         eg: maybe some attribute is not available and usually you'd return None. There is no strict type
    ///         checking here, so raise an error and bail instead.
    def stock_reagent(
        &self,
        cost: Int,
        type_in: &PyObject,
        variations_in: &PySequence,
        type_out: &PyObject,
        variations_out: &PySequence,
        function: PyObject
    ) -> PyResult<PyObject> {
        let hash_in = type_in.hash(py)?;
        let hash_out = type_out.hash(py)?;
        let hash_func = function.hash(py)?;
        let hash_var_in = hash_seq!(py, variations_in);
        let hash_var_out = hash_seq!(py, variations_out);

        // Store a reference to the python object in this outer layer
        // but refer to it via its hash.
        self.functions(py).borrow_mut().insert(hash_func, function);
        self.graph(py).borrow_mut().add_edge(cost, hash_in, hash_var_in, hash_out, hash_var_out, hash_func);
        Ok(py.None())
    }

    /// Supply a function that will attempt to apply initial variations automatically.
    /// This is a convenience aid, to assist in detecting inputs automatically so they do not
    /// need to be expicitly specified.
    /// The detector should run quickly so as to keep the entire process smooth.
    /// ie simple attribute checks, string regex etc
    ///
    /// Args:
    ///     type_in:
    ///         The type of input this detector accepts.
    ///     function:
    ///         Function that takes the value provided (of the type above) and yields any variations it finds.
    ///         eg: str type could check for link type if the string is http://something.html and
    ///         yield "protocol" "http"
    def stock_activator(&self, type_in: &PyObject, function: PyObject) -> PyResult<PyObject> {
        self.activators(py).borrow_mut().entry(type_in.hash(py)?).or_insert(Vec::new()).push(function);
        Ok(py.None())
    }

    /// From a given type, attempt to produce a requested type.
    /// OR from some given data, attempt to traverse links to get the requested data.
    ///
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
        type_want: &PyObject,
        variations_want: Option<&PySequence> = None,
        type_have: Option<&PyObject> = None,
        variations_have: Option<&PySequence> = None,
        explicit: bool = false
    ) -> PyResult<PyObject> {
        let hash_in = match type_have {
            Some(type_override) => type_override.hash(py)?,
            None => value.get_type(py).into_object().hash(py)?
        };
        let hash_out = type_want.hash(py)?;
        let hash_var_out = match variations_want {
            Some(vars) => hash_seq!(py, vars),
            None => BTreeSet::new(),
        };
        let mut hash_var_in = match variations_have {
            Some(vars) => hash_seq!(py, vars),
            None => BTreeSet::new(),
        };
        if !explicit {
            // We don't want to be explicit, so
            // run the activator to detect initial variations
            if let Some(funcs) = self.activators(py).borrow().get(&hash_in) {
                for func in funcs {
                    let variations = PyIterator::from_object(py, func.call(py, (value.clone_ref(py),), None)?)?;
                    for variation in variations {
                        hash_var_in.insert(variation?.hash(py)?);
                    }
                }
            }
        }
        println!(">> {:?}", hash_var_in);

        // Retry a few times, if something breaks along the way.
        // Collect errors.
        // If we run out of paths to take or run out of reties,
        // and there are still errors. Raise with info from all of them.
        let mut skip_edges = BTreeSet::new();
        let mut errors = Vec::new();
        'outer: for _ in 0..10 {
            if let Some(edges) = self.graph(py).borrow().search(hash_in, &hash_var_in, hash_out, &hash_var_out, &skip_edges) {
                let functions = self.functions(py).borrow();
                let mut result = value.clone_ref(py);
                for edge in edges {
                    let func = functions.get(&edge.hash_func).expect("Function is there");
                    match func.call(py, (result,), None) {
                        Ok(res) => result = res,
                        Err(mut err) => {
                            errors.push(
                                format!(
                                    "{}: {}",
                                    err.get_type(py).name(py),
                                    err.instance(py).str(py)?.to_string(py)?,
                                )
                            );
                        // Ignore these when trying again.
                        // This allows some level of failure
                        // and with enough edges perhaps we
                        // can find another path.
                        skip_edges.insert(edge);
                        continue 'outer
                        }
                    };
                }
                return Ok(result)
            }
            break
        }
        if errors.len() != 0 {
            Err(PyErr::new::<CommandFailure, _>(py, format!(
                "Some problems occurred during the transmution process:\n{}",
                errors.join("\n")
                )))
        } else {
            Err(PyErr::new::<LackingReagentFailure, _>(
                py, "Could not create a transmutation. Missing some critical reagents. Consider adding more."))
        }
    }

    ///////////////////////////////////////////////////////////////
    // Satisfy python garbage collector
    // because we hold a reference to some functions provided
    def __traverse__(&self, visit) {
        for function in self.functions(py).borrow().values() {
            visit.call(function)?;
        }
        for functions in self.activators(py).borrow().values() {
            for function in functions {
                visit.call(function)?;
            }
        }
        Ok(())
    }

    def __clear__(&self) {
        for (_, func) in self.functions(py).borrow_mut().drain() {
            func.release_ref(py);
        }
        for (_, mut funcs) in self.activators(py).borrow_mut().drain() {
            for func in funcs.drain(..) {
                func.release_ref(py);
            }
        }
    }
    ///////////////////////////////////////////////////////////////
});
