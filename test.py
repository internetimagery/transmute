import sys

sys.path.append("target/release")

import transmute

grimoire = transmute.Grimoire()
grimoire.inscribe_transmutation(1, str, int)
grimoire.transmute("123", int)
print(grimoire)
