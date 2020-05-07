import unittest
import sys, itertools

sys.path.append("target/release")
from transmute import Grimoire, NoChainError

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


class TestGrimoire(unittest.TestCase):
    def setUp(self):
        self.grimoire = Grimoire()

    def test_basic_graph(self):
        # A - B - C - D
        # |         /
        # E - F - G

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_E, [], AtoE())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.grimoire.inscribe_transmutation(1, TYPE_E, [], TYPE_F, [], EtoF())
        self.grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_G, [], FtoG())
        self.grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_D, [], GtoD())

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_D, [], TYPE_A),
            "start -> AtoB -> BtoC -> CtoD",
        )

    def test_join_graph(self):
        # A           E
        #  \         /
        #   - C - D -
        #  /         \
        # B           F

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_C, [], AtoC())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_E, [], DtoE())
        self.grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_F, [], DtoF())

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_F, [], TYPE_A),
            "start -> AtoC -> CtoD -> DtoF",
        )

    def test_basic_variation(self):

        # A = B = C'

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_A, [], BtoA())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.grimoire.inscribe_transmutation(
            1, TYPE_C, [], TYPE_B, ["var"], CtoB("var")
        )

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_A, ["var"], TYPE_A),
            "start -> AtoB -> BtoC -> CtoB:var -> BtoA",
        )

    def test_variation_preference(self):
        #     B       D'
        #    / \     / \
        # A -   - C -   - E
        #    \ /     \ /
        #     F'      G

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_F, [], AtoF())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.grimoire.inscribe_transmutation(
            2, TYPE_C, [], TYPE_D, ["var2"], CtoD("var2")
        )
        self.grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_G, [], CtoG())
        self.grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_E, [], DtoE())
        self.grimoire.inscribe_transmutation(
            1, TYPE_F, [], TYPE_C, ["var1"], FtoC("var1")
        )
        self.grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_E, [], GtoE())

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_E, ["var1", "var2"], TYPE_A),
            "start -> AtoF -> FtoC:var1 -> CtoD:var2 -> DtoE",
        )

    def test_revisit(self):

        # A - B - C - D'
        #  \  |   |   |
        #   - E - F - G

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_C, [], BtoC())
        self.grimoire.inscribe_transmutation(1, TYPE_B, [], TYPE_E, [], BtoE())
        self.grimoire.inscribe_transmutation(
            3, TYPE_C, [], TYPE_D, ["var"], CtoD("var")
        )
        self.grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_F, [], CtoF())
        self.grimoire.inscribe_transmutation(1, TYPE_D, [], TYPE_G, [], DtoG())
        self.grimoire.inscribe_transmutation(1, TYPE_E, [], TYPE_A, [], EtoA())
        self.grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_E, [], FtoE())
        self.grimoire.inscribe_transmutation(1, TYPE_G, [], TYPE_F, [], GtoF())

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_A, ["var"], TYPE_A),
            "start -> AtoB -> BtoC -> CtoD:var -> DtoG -> GtoF -> FtoE -> EtoA",
        )

    def test_failures(self):

        # A - B
        # C - D
        # E'- F - G!

        self.grimoire.inscribe_transmutation(1, TYPE_A, [], TYPE_B, [], AtoB())
        self.grimoire.inscribe_transmutation(1, TYPE_C, [], TYPE_D, [], CtoD())
        self.grimoire.inscribe_transmutation(
            1, TYPE_E, ["var"], TYPE_F, [], EtoF("var")
        )
        self.grimoire.inscribe_transmutation(1, TYPE_F, [], TYPE_G, [], bad_transmuter)

        self.assertEqual(
            self.grimoire.transmute("start", TYPE_F, [], TYPE_E, ["var"]),
            "start -> EtoF:var",
        )
        # except NoChainError:
        with self.assertRaises(NoChainError):
            self.grimoire.transmute("start", TYPE_D, [], TYPE_A)

        # except NoTransmuterError:
        with self.assertRaises(Exception):
            self.grimoire.transmute("start", TYPE_F, [], TYPE_E)

        # except ExecutionError:
        with self.assertRaises(Exception):
            self.grimoire.transmute("start", TYPE_G, [], TYPE_F)


if __name__ == "__main__":
    unittest.main()
