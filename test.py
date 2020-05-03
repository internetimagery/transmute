
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
grimoire.inscribe_transmutation(1, TYPE_A, TYPE_B, AtoB())
grimoire.inscribe_transmutation(1, TYPE_A, TYPE_E, AtoE())
grimoire.inscribe_transmutation(1, TYPE_B, TYPE_C, BtoC())
grimoire.inscribe_transmutation(1, TYPE_C, TYPE_D, CtoD())
grimoire.inscribe_transmutation(1, TYPE_E, TYPE_F, EtoF())
grimoire.inscribe_transmutation(1, TYPE_F, TYPE_G, FtoG())
grimoire.inscribe_transmutation(1, TYPE_G, TYPE_D, GtoD())

assert (
    grimoire.transmute("start", TYPE_D, TYPE_A)
    == "start -> AtoB -> BtoC -> CtoD"
)

# A           E
#  \         /
#   - C - D -
#  /         \
# B           F

grimoire = Grimoire()
grimoire.inscribe_transmutation(1, TYPE_A, TYPE_C, AtoC())
grimoire.inscribe_transmutation(1, TYPE_B, TYPE_C, BtoC())
grimoire.inscribe_transmutation(1, TYPE_C, TYPE_D, CtoD())
grimoire.inscribe_transmutation(1, TYPE_D, TYPE_E, DtoE())
grimoire.inscribe_transmutation(1, TYPE_D, TYPE_F, DtoF())

assert (
    grimoire.transmute("start", TYPE_F, TYPE_A)
    == "start -> AtoC -> CtoD -> DtoF"
)
