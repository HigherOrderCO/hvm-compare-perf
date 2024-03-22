# To format and tabulate data
import pandas as pd

import math

import matplotlib.pyplot as plt
# To correctly parse SI abbreviations.
import pint

ureg = pint.UnitRegistry()

data = pd.read_csv("perf.csv")

data["file"] = data["file"].apply(lambda x: x.replace("./programs/", "").replace(".hvmc", ""))
data["hash"] = data["hash"].apply(lambda x: x.replace("compare-", ""))
data["time"] = data["time"].apply(lambda x: ureg.Quantity(x))
data["time"] = data["time"].apply(lambda x: x.to(ureg.s).m if not x.dimensionless else float("nan"))
data["rwts"] = data["rwts"].apply(lambda x: float(x.replace("_","")) if type(x) == str else x)
data["rwps"] = data["rwps"].apply(lambda x: float(x.replace("M","").replace("m","").strip())*10**6 if type(x) == str else x)
index = pd.MultiIndex.from_frame(data[["hash", "file", "mode"]], names = ["hash", "file", "mode"])
data = pd.Series([tuple(row) for row in data[["rwts", "rwps", "time"]].to_numpy()], index = index)
data = pd.DataFrame(list(data), columns = ["rwts", "rwps", "time"], index = index)
data = data.stack(future_stack=True)
data = data.unstack(level="hash")
data = data.xs("time", level=2)
data = data.drop("c2")

def pretty_plaintext_format(data) -> str:
	s = ""
	s += f"{'file':11} {'mode':11} {'stat':4} "
	for k in data.columns:
		s += f"{k[:11]:11} "
	s += "\n"
	s += "=" * 100 + "\n"

	old_k = None

	for (k, v) in data.iterrows():
		k = (*k, "time")
		if old_k:
			if old_k[0] != k[0]:
				s += "=" * 100 + "\n"
			elif old_k[1] != k[1]:
				s += ""#"-" * 100 + "\n"
		if not old_k or old_k[0] != k[0]:
			s += f"{k[0][:11]:11} "
		else:
			s += f" " * 12
		if not old_k or old_k[1] != k[1]:
			s += f"{k[1][:11]:11} "
		else:
			s += f" " * 12
		if not old_k or old_k[2] != k[2]:
			s += f"{k[2][:4]:4} "
		else:
			s += f" " * 5

		if k[2] == "rwps":
			# higher is better
			key = lambda i: -v.iloc[i]
		else:
			key = lambda i: float("inf") if math.isnan(v.iloc[i]) else v.iloc[i]
		order = list(sorted(range(5), key = key))
		for idx, i in enumerate(v):
			s += [
				"\x1b[1;32m",
				"\x1b[32m",
				"",
				"\x1b[31m",
				"\x1b[1;31m"
			][order.index(idx)]
			if k[2] == "rwts":
				s += f"{i / 10**6:9.3f} M "
			elif k[2] == "rwps":
				s += f"{i / 10**6:9.3f} M "
			elif k[2] == "time":
				s += f"{i:9.3f} s "
			s += "\x1b[0m"
		s += "\n"
		old_k = k

	return s


print(pretty_plaintext_format(data))