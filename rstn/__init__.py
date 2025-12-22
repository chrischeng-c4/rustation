"""rstn v2 - Rustation Development Toolkit.

Built following State-First MVI architecture principles:
- All state must be JSON/YAML serializable
- UI = render(State) - pure functions
- Event → AppMsg → reduce → (State, Effects) → EffectExecutor
"""

from rstn.effect import AppEffect
from rstn.msg import AppMsg
from rstn.reduce import reduce
from rstn.state import AppState

__version__ = "0.1.0"

__all__ = ["AppState", "AppMsg", "AppEffect", "reduce", "__version__"]
