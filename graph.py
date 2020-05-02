# Consider multiple inputs dependency graph?
# a type would be represented by the Type + Variations
# each node have multiple (positional) inputs and a single output

# to do it?
# nodes would need to know about their children as well as parent
# every time a node hits an input (from backwards search), walk back and see if dependencies are satisfied
# eg walk the chain as we are doing now, until reaching an end. If the end is the goal walk back to the next node missing
# a dependency. If it is not a goal, the chain is bust, so walk back to the nearest junction and clear that chain.


# Copyright 2020 Jason Dixon
# Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
# The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

from __future__ import division

try:
    from typing import *
except ImportError:
    pass

from collections import namedtuple, defaultdict
from heapq import heappop, heappush
from itertools import chain
from traceback import format_exc
from logging import getLogger

LOG = getLogger(__name__)

__all__ = ["Grimoire"]


class TransmuteError(Exception):
    """ Base error class coming from the transmuter """


class NoTransmuterError(TransmuteError):
    """ Raised when no transmuter can be found for the requested types.
        Different from chain error. In this case not one transmuter can
        satisfy the input and/or output.
    """


class NoChainError(TransmuteError):
    """ Raised when a chain of transmuters could not be discovered.
        Its likely more transmuters are needed to fill the gaps.
    """


class ExecutionError(TransmuteError):
    """ Raised when nodes fail to execute during runtime.
    """


Transmuter = namedtuple(
    "Transmuter",
    ("cost", "hash_in", "hash_out", "var_hash_in", "var_hash_out", "function"),
)  # type: NamedTuple[int, int, int, FrozenSet[int], FrozenSet[int], Callable[[Any], Any]]
Node = namedtuple(
    "Node", ("adjusted_cost", "cost", "transmuter", "parent", "variations")
)  # type: NamedTuple[float, int, Transmuter, Optional[Node], FrozenSet[int]]


class Grimoire(object):
    """ Simple network housing a collection of equally simple "a to b" functions.
        Providing the ability to chain a bunch of them together for more complex transmutations.

        You could be wanting to transmute between a chain of types, or traverse a bunch of object oriented links.
        If you're often thinking "I have this, how can I get that", then this type of solution could help.

        >>> grimoire = Grimoire()
        >>> grimoire.inscribe_transmutation(1, str, ["href"], WebPage, [], load_webpage)
        >>> grimoire.inscribe_detector(str, http_detector)
        >>> grimoire.transmute("http://somewhere.html", WebPage)
    """

    def __init__(self):
        self._input_map = defaultdict(list)  # type: DefaultDict[int, List[Transmuter]]
        self._output_map = defaultdict(list)  # type: DefaultDict[int, List[Transmuter]]
        self._detector_maps = defaultdict(
            list
        )  # type: DefaultDict[Any, List[Callable[[Any], Iterator[Any]]]]

    def inscribe_transmutation(
        self, cost, type_in, variations_in, type_out, variations_out, function
    ):  # type: (int, Any, Sequence[Any], Any, Sequence[Any], Callable[[Any], Any]) -> None
        """ Write a function into the grimoire so it may be used as a piece in the transmutation chain later.
            Eventually a transmutation chain will consist of a number of these placed back to back.
            So the simpler and smaller the transmutation the better.

            Args:
                cost:
                    A number representing how much work this transmuter needs to do.
                    Lower numbers are prioritized.
                    eg: just getting an attribute would be a low number. Accessing an http service would be higher etc
                type_in:
                    Type of input expected.
                    Typically a class type, but can be anything hashable that is relevant to a workflow.
                    eg str / MyClass or a composite type eg frozenset([Type1, Type2])
                variations_in:
                    A sequence of hashable "tags" further describing the input type.
                    For the node to be used, all these variations are required (dependencies).
                    This is useful if the more simple type is not enough by itself.
                    eg: str (can be path/href/name/any concept)
                type_out:
                    Same as "type_in", but representing the output of the transmutation.
                    NOTE: it is important the transmuter only outputs the stated type (eg no None option)
                variations_out:
                    Same as "variations_in" except that variations are descriptive and not dependencies.
                    They can satisfy dependencies for transmuters further down the chain.
                function:
                    The transmuter itself. Take a single input, produce a single output.
                    It is important that only an simple transmutation is made, and that any deviation is raised as an Error.
                    eg: maybe some attribute is not available and usually you'd return None. There is no strict type
                    checking here, so raise an error and bail instead.
        """
        hash_in = hash(type_in)
        hash_out = hash(type_out)
        hash_var_in = frozenset(hash(v) for v in variations_in)
        hash_var_out = frozenset(hash(v) for v in variations_out)
        transmuter = Transmuter(
            cost, hash_in, hash_out, hash_var_in, hash_var_out, function
        )
        heappush(self._input_map[hash_in], transmuter)
        heappush(self._output_map[hash_out], transmuter)

    def inscribe_detector(
        self, type_, detector
    ):  # type: (Any, Callable[[Any], Iterator[Any]]) -> None
        """ Supply a function that will attempt to apply initial variations automatically.
            This is a convenience aid, to assist in detecting inputs automatically so they do not
            need to be expicitly specified.
            The detector should run quickly so as to keep the entire process smooth.
            ie simple attribute checks, string regex etc

            Args:
                type_:
                    The type of input this detector accepts.
                detector:
                    Function that takes the value provided (of the type above) and yields any variations it finds.
                    eg: str type could check for link type if the string is http://something.html
        """
        self._detector_maps[type_].append(detector)

    def transmute(
        self,
        value,
        type_want,
        variations_want=None,
        type_have=None,
        variations_have=None,
        explicit=False,
    ):  # type: (Any, Any, Optional[Sequence[Any]], Optional[Any], Optional[Sequence[Any]], bool) -> Any
        """ From a given type, attempt to produce a requested type.
            OR from some given data, attempt to traverse links to get the requested data.

            Args:
                value: The input you have going into the process. This can be anything.
                type_want:
                    The type you want to recieve. A chain of transmuters will be produced
                    attempting to attain this type.
                variations_want:
                    A sequence of variations further describing the type you wish to attain.
                    This is optional but can help guide a transmutation through more complex types.
                type_have:
                    An optional override for the starting type.
                    If not provided the type of the value is taken instead.
                variations_have:
                    Optionally include any extra variations to the input.
                    If context is known but hard to detect this can help direct a more complex
                    transmutation.
                explicit:
                    If this is True, the variations_have attribute will entirely override
                    any detected tags. Use this if the detection is bad and you know EXACTLY what you need.

        """
        if type_have is None:
            type_have = type(value)
        if not explicit:
            detected_variations = (
                variation
                for detector in self._detector_maps.get(type_have, [])
                for variation in detector(value)
            )
            variations_have = list(chain(variations_have or [], detected_variations))

        ignore_transmuters = set()  # type: Set[Transmuter]
        errors = []  # type: List[Tuple[str, str, str]]
        for _ in range(10):  # Number of retries
            transmuters = self._search(
                type_have,
                variations_have or [],
                type_want,
                variations_want or [],
                ignore_transmuters,
            )
            if not transmuters:
                if errors:
                    break
                raise NoChainError(
                    "Could not transmute {} to {}".format(type_have, type_want)
                )
            LOG.debug("Resolving chain: %s", transmuters)
            try:
                result = value
                for transmuter in transmuters:
                    result = transmuter.function(result)
            except Exception as err:
                errors.append((type(err).__name__, str(err), format_exc()))
                ignore_transmuters.add(transmuter)
            else:
                if errors:
                    LOG.warning(
                        "The following errors were raised during execution:\n%s"
                        "\n".join(
                            "{}: {}\n---\n{}---".format(*error) for error in errors
                        )
                    )
                return result
        raise ExecutionError(
            "The following errors were raised during execution:\n{}".format(
                "\n".join("{}: {}\n---\n{}---".format(*error) for error in errors)
            )
        )

    def _search(
        self, type_in, variations_in, type_out, variations_out, ignore_transmuters
    ):  # type: (Any, Any, Sequence[Any], Sequence[Any], Set[Transmuter]) -> List[Transmuter]
        """ Search through the provided transmuters and attempt to build an optimal chain linking them.
            Chains are built by linking the same types (out -> in) between transmuters.
            Variations are considered dependencies, and are one time use. ie spent when reaching a node that
            requires them.
            Transmuters that have a lower cost are prioritized for searching.
            Transmuters that satisfy more variations are prioritized also.
            Transmuters that have variations that cannot be provided are avoided.
        """
        in_hash = hash(type_in)
        out_hash = hash(type_out)
        in_variations_hash = frozenset(hash(v) for v in variations_in)
        out_variations_hash = frozenset(hash(v) for v in variations_out)

        in_queue = [
            Node(
                trans.cost / (len(trans.var_hash_in) + 1),
                trans.cost,
                trans,
                None,
                (in_variations_hash - trans.var_hash_in) | trans.var_hash_out,
            )
            for trans in self._input_map.get(in_hash, [])
            if trans.var_hash_in <= in_variations_hash  # requirement check
        ]
        if not in_queue:
            raise NoTransmuterError(
                "No transmuter exists with input type for {}".format(type_in)
            )
        out_queue = [
            Node(
                trans.cost / (len(out_variations_hash & trans.var_hash_out) + 1),
                trans.cost,
                trans,
                None,
                (out_variations_hash - trans.var_hash_out) | trans.var_hash_in,
            )
            for trans in self._output_map.get(out_hash, [])
        ]
        if not out_queue:
            raise NoTransmuterError(
                "No transmuter exists with output type for {}".format(type_out)
            )

        in_visited = defaultdict(
            dict
        )  # type: DefaultDict[Transmuter, Dict[FrozenSet[int], Node]]
        out_visited = defaultdict(
            dict
        )  # type: DefaultDict[Transmuter, Dict[FrozenSet[int], Node]]

        while in_queue or out_queue:

            ###################
            # Search forwards #
            ###################
            if (in_queue and len(in_queue) < len(out_queue)) or not out_queue:
                node = heappop(in_queue)

                # Ignore transmuters flagged
                if node.transmuter in ignore_transmuters:
                    continue

                # Check if the output type equals our goal output
                # Also check that the goal variations are all present
                if (
                    node.transmuter.hash_out == out_hash
                    and out_variations_hash <= node.variations
                ):
                    # We have reached our goal, and satisfied the depencency check!
                    return [n.transmuter for n in reversed(list(self._walk_node(node)))]

                # Check if we have intersected with the opposing search
                for out_node in out_visited[node.transmuter].values():
                    # We have intersected a path traveling from the other direction.
                    # We reached our goal! Pending dependency check.
                    if out_node.variations <= (
                        node.parent.variations if node.parent else in_variations_hash
                    ):
                        return [
                            n.transmuter
                            for n in chain(
                                reversed(list(self._walk_node(node))[1:]),
                                self._walk_node(out_node),
                            )
                        ]

                # Add new nodes to the queue, contiue our search
                in_visited[node.transmuter][
                    node.parent.variations if node.parent else None
                ] = node
                for transmuter in self._input_map.get(node.transmuter.hash_out, []):
                    if node.variations in in_visited[transmuter]:
                        continue
                    if not transmuter.var_hash_in <= node.variations:
                        # dependency check
                        continue
                    # A ----------->
                    # B ---|B| |
                    # C ---|C|D|--->
                    variations = (
                        node.variations - transmuter.var_hash_in
                    ) | transmuter.var_hash_out
                    heappush(
                        in_queue,
                        Node(
                            node.cost
                            + transmuter.cost / (len(transmuter.var_hash_in) + 1),
                            node.cost + transmuter.cost,
                            transmuter,
                            node,
                            node.variations - transmuter.var_hash_in,
                        ),
                    )
            ####################
            # Search backwards #
            ####################
            elif out_queue:
                node = heappop(out_queue)

                # Ignore transmuters flagged
                if node.transmuter in ignore_transmuters:
                    continue

                # Check the input of the node matches the requested input.
                # Also check the nodes variations all are provided by the input.
                if (
                    node.transmuter.hash_in == in_hash
                    and node.variations <= in_variations_hash
                ):
                    # We reached our goal backwards and satisfied dependency check!
                    return [n.transmuter for n in self._walk_node(node)]

                # Check with the forward search and see if we have intersected at all
                for in_node in in_visited[node.transmuter].values():
                    # We have intersected a path traveling from the other direction.
                    # Check dependencies, we may have reached our goal
                    if node.variations <= (
                        in_node.parent.variations
                        if in_node.parent
                        else in_variations_hash
                    ):
                        return [
                            n.transmuter
                            for n in chain(
                                reversed(list(self._walk_node(in_node))[1:]),
                                self._walk_node(node),
                            )
                        ]

                # Continue our search, appending new nodes to the queue
                out_visited[node.transmuter][
                    node.parent.variations if node.parent else None
                ] = node
                for transmuter in self._output_map.get(node.transmuter.hash_in, []):
                    if node.variations in out_visited[transmuter]:
                        continue
                    if not transmuter.var_hash_out <= node.variations:
                        # Reverse dependency check
                        continue
                    # A <-----------
                    # B <---|B| |
                    # C <---|C|D|--- D
                    variations = (
                        node.variations - transmuter.var_hash_out
                    ) | transmuter.var_hash_in
                    heappush(
                        out_queue,
                        Node(
                            node.cost
                            + transmuter.cost
                            / (len(transmuter.var_hash_out & node.variations) + 1),
                            node.cost + transmuter.cost,
                            transmuter,
                            node,
                            variations,
                        ),
                    )

        return []

    @staticmethod
    def _walk_node(node):  # type: (Node) -> Iterator[Node]
        while node:
            yield node
            node = node.parent


if __name__ == "__main__":

    import sys, itertools

    module = sys.modules[__name__]

    class Executor(object):
        def __init__(self, variation=""):
            self.variation = variation and ":" + variation

        def __call__(self, value):
            return value + " -> " + self.__class__.__name__ + self.variation

    letters = "ABCDEFG"

    for letter in letters:
        setattr(module, "TYPE_" + letter, type("TYPE_" + letter, (Executor,), {}))

    for a, b in itertools.permutations(letters, 2):
        setattr(module, a + "to" + b, type(a + "to" + b, (Executor,), {}))

    def bad_transmuter(_):
        raise RuntimeError("BAD STUFF")

    # A - B - C - D
    # |         /
    # E - F - G

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_E, [], AtoE())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
    grimoire.inscribe_transmutation(1, TYPE_E, [], TYPE_F, [], EtoF())
    grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_G, [], FtoG())
    grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_D, [], GtoD())

    assert (
        grimoire.transmute("start", TYPE_D, [], TYPE_A)
        == "start -> AtoB -> BtoC -> CtoD"
    )

    # A           E
    #  \         /
    #   - C - D -
    #  /         \
    # B           F

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_C, [], AtoC())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
    grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_E, [], DtoE())
    grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_F, [], DtoF())

    assert (
        grimoire.transmute("start", TYPE_F, [], TYPE_A)
        == "start -> AtoC -> CtoD -> DtoF"
    )

    # A = B = C'

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_A, [], BtoA())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_B, ["var"], CtoB("var"))

    assert (
        grimoire.transmute("start", TYPE_A, ["var"], TYPE_A)
        == "start -> AtoB -> BtoC -> CtoB:var -> BtoA"
    )

    #     B       D'
    #    / \     / \
    # A -   - C -   - E
    #    \ /     \ /
    #     F'      G

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_F, [], AtoF())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
    grimoire.inscribe_transmutation(2, TYPE_C, [], TYPE_D, ["var2"], CtoD("var2"))
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_G, [], CtoG())
    grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_E, [], DtoE())
    grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_C, ["var1"], FtoC("var1"))
    grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_E, [], GtoE())

    assert (
        grimoire.transmute("start", TYPE_E, ["var1", "var2"], TYPE_A)
        == "start -> AtoF -> FtoC:var1 -> CtoD:var2 -> DtoE"
    )

    # A - B - C - D'
    #  \  |   |   |
    #   - E - F - G

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
    grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_E, [], BtoE())
    grimoire.inscribe_transmutation(3, TYPE_C, [], TYPE_D, ["var"], CtoD("var"))
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_F, [], CtoF())
    grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_G, [], DtoG())
    grimoire.inscribe_transmutation(1, TYPE_E, [], TYPE_A, [], EtoA())
    grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_E, [], FtoE())
    grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_F, [], GtoF())

    assert (
        grimoire.transmute("start", TYPE_A, ["var"], TYPE_A)
        == "start -> AtoB -> BtoC -> CtoD:var -> DtoG -> GtoF -> FtoE -> EtoA"
    )

    # A - B
    # C - D
    # E'- F - G!

    grimoire = Grimoire()
    grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
    grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
    grimoire.inscribe_transmutation(1, TYPE_E, ["var"], TYPE_F, [], EtoF("var"))
    grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_G, [], bad_transmuter)

    assert (
        grimoire.transmute("start", TYPE_F, [], TYPE_E, ["var"]) == "start -> EtoF:var"
    )
    try:
        grimoire.transmute("start", TYPE_D, [], TYPE_A)
    except NoChainError:
        pass
    else:
        assert False

    try:
        grimoire.transmute("start", TYPE_F, [], TYPE_E)
    except NoTransmuterError:
        pass
    else:
        assert False

    try:
        grimoire.transmute("start", TYPE_G, [], TYPE_F)
    except ExecutionError:
        pass
    else:
        assert False
