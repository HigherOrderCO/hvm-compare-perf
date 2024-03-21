# To format and tabulate data
import pandas as pd

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
pd.set_option('display.max_colwidth',1000)
pd.set_option('display.max_columns', 7)
import sys
data.to_csv(sys.stdout)