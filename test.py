import unittest
import sys, itertools

sys.path.append("target/release")
from transmute import Lab, LackingReagentFailure, CommandFailure

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


def activator(value):
    yield "var"


class TestLab(unittest.TestCase):
    def setUp(self):
        self.lab = Lab()

    def test_basic_graph(self):
        # A - B - C - D
        # |         /
        # E - F - G

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_A, [], TYPE_E, [], AtoE())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.lab.stock_reagent(1, TYPE_E, [], TYPE_F, [], EtoF())
        self.lab.stock_reagent(1, TYPE_F, [], TYPE_G, [], FtoG())
        self.lab.stock_reagent(1, TYPE_G, [], TYPE_D, [], GtoD())

        self.assertEqual(
            self.lab.transmute("start", TYPE_D, [], TYPE_A),
            "start -> AtoB -> BtoC -> CtoD",
        )

    def test_activator(self):
        # A - B - C
        #  \     /
        #   - D'-

        self.lab.stock_activator(TYPE_A, activator)
        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_A, ["var"], TYPE_D, [], AtoD("var"))
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(1, TYPE_D, [], TYPE_C, [], DtoC())

        self.assertEqual(
            self.lab.transmute("start", TYPE_C, [], TYPE_A),
            "start -> AtoD:var -> DtoC",
        )


    def test_join_graph(self):
        # A           E
        #  \         /
        #   - C - D -
        #  /         \
        # B           F

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_C, [], AtoC())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.lab.stock_reagent(1, TYPE_D, [], TYPE_E, [], DtoE())
        self.lab.stock_reagent(1, TYPE_D, [], TYPE_F, [], DtoF())

        self.assertEqual(
            self.lab.transmute("start", TYPE_F, [], TYPE_A),
            "start -> AtoC -> CtoD -> DtoF",
        )

    def test_basic_variation(self):

        # A = B = C'

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_A, [], BtoA())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(
            1, TYPE_C, [], TYPE_B, ["var"], CtoB("var")
        )

        self.assertEqual(
            self.lab.transmute("start", TYPE_A, ["var"], TYPE_A),
            "start -> AtoB -> BtoC -> CtoB:var -> BtoA",
        )

    def test_variation_preference(self):
        #     B       D'
        #    / \     / \
        # A -   - C -   - E
        #    \ /     \ /
        #     F'      G

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_A, [], TYPE_F, [], AtoF())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(
            2, TYPE_C, [], TYPE_D, ["var2"], CtoD("var2")
        )
        self.lab.stock_reagent(1, TYPE_C, [], TYPE_G, [], CtoG())
        self.lab.stock_reagent(1, TYPE_D, [], TYPE_E, [], DtoE())
        self.lab.stock_reagent(
            1, TYPE_F, [], TYPE_C, ["var1"], FtoC("var1")
        )
        self.lab.stock_reagent(1, TYPE_G, [], TYPE_E, [], GtoE())

        self.assertEqual(
            self.lab.transmute("start", TYPE_E, ["var1", "var2"], TYPE_A),
            "start -> AtoF -> FtoC:var1 -> CtoD:var2 -> DtoE",
        )

    def test_revisit(self):

        # A - B - C - D'
        #  \  |   |   |
        #   - E - F - G

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.lab.stock_reagent(1, TYPE_B, [], TYPE_E, [], BtoE())
        self.lab.stock_reagent(
            3, TYPE_C, [], TYPE_D, ["var"], CtoD("var")
        )
        self.lab.stock_reagent(1, TYPE_C, [], TYPE_F, [], CtoF())
        self.lab.stock_reagent(1, TYPE_D, [], TYPE_G, [], DtoG())
        self.lab.stock_reagent(1, TYPE_E, [], TYPE_A, [], EtoA())
        self.lab.stock_reagent(1, TYPE_F, [], TYPE_E, [], FtoE())
        self.lab.stock_reagent(1, TYPE_G, [], TYPE_F, [], GtoF())

        self.assertEqual(
            self.lab.transmute("start", TYPE_A, ["var"], TYPE_A),
            "start -> AtoB -> BtoC -> CtoD:var -> DtoG -> GtoF -> FtoE -> EtoA",
        )

    def test_failures(self):

        # A - B
        # C - D
        # E'- F - G!

        self.lab.stock_reagent(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.lab.stock_reagent(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.lab.stock_reagent(
            1, TYPE_E, ["var"], TYPE_F, [], EtoF("var")
        )
        self.lab.stock_reagent(1, TYPE_F, [], TYPE_G, [], bad_transmuter)

        self.assertEqual(
            self.lab.transmute("start", TYPE_F, [], TYPE_E, ["var"]),
            "start -> EtoF:var",
        )
        with self.assertRaises(LackingReagentFailure):
            self.lab.transmute("start", TYPE_D, [], TYPE_A)

        with self.assertRaises(LackingReagentFailure):
            self.lab.transmute("start", TYPE_F, [], TYPE_E)

        with self.assertRaises(CommandFailure):
            self.lab.transmute("start", TYPE_G, [], TYPE_F)


if __name__ == "__main__":
    unittest.main()
