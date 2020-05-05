
import sys, itertools
sys.path.append("target/release")
from transmute import Grimoire


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

