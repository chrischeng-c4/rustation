"""Performance tests for rstn v2.

Verifies that core operations meet performance requirements:
- State serialization < 10ms
- Reduce operation < 1ms
"""

from __future__ import annotations

import time

from rstn.msg import KeyPressed, Noop, SwitchView, Tick
from rstn.reduce import reduce
from rstn.state import AppState, Command, ViewType, WorktreeViewState


class TestSerializationPerformance:
    """Tests for state serialization performance."""

    def test_serialize_default_state(self) -> None:
        """Test serializing default state is fast."""
        state = AppState()

        start = time.perf_counter()
        for _ in range(100):
            _ = state.model_dump_json()
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 100) * 1000
        assert avg_ms < 10, f"Serialization took {avg_ms:.2f}ms (expected <10ms)"

    def test_serialize_state_with_commands(self) -> None:
        """Test serializing state with commands is fast."""
        commands = [
            Command(id=f"cmd_{i}", label=f"Command {i}", description=f"Description {i}")
            for i in range(20)
        ]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        start = time.perf_counter()
        for _ in range(100):
            _ = state.model_dump_json()
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 100) * 1000
        assert avg_ms < 10, f"Serialization took {avg_ms:.2f}ms (expected <10ms)"

    def test_deserialize_state(self) -> None:
        """Test deserializing state is fast."""
        state = AppState()
        json_str = state.model_dump_json()

        start = time.perf_counter()
        for _ in range(100):
            _ = AppState.model_validate_json(json_str)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 100) * 1000
        assert avg_ms < 10, f"Deserialization took {avg_ms:.2f}ms (expected <10ms)"

    def test_roundtrip_serialization(self) -> None:
        """Test full serialization roundtrip is fast."""
        commands = [
            Command(id=f"cmd_{i}", label=f"Command {i}")
            for i in range(10)
        ]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        start = time.perf_counter()
        for _ in range(50):
            json_str = state.model_dump_json()
            _ = AppState.model_validate_json(json_str)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 50) * 1000
        assert avg_ms < 20, f"Roundtrip took {avg_ms:.2f}ms (expected <20ms)"


class TestReducePerformance:
    """Tests for reducer performance."""

    def test_reduce_noop(self) -> None:
        """Test reduce with Noop is fast."""
        state = AppState()
        msg = Noop()

        start = time.perf_counter()
        for _ in range(1000):
            new_state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 1000) * 1000
        assert avg_ms < 1, f"Noop reduce took {avg_ms:.3f}ms (expected <1ms)"

    def test_reduce_tick(self) -> None:
        """Test reduce with Tick is fast."""
        state = AppState()
        msg = Tick()

        start = time.perf_counter()
        for _ in range(1000):
            new_state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 1000) * 1000
        assert avg_ms < 1, f"Tick reduce took {avg_ms:.3f}ms (expected <1ms)"

    def test_reduce_switch_view(self) -> None:
        """Test reduce with SwitchView is fast."""
        state = AppState()

        start = time.perf_counter()
        for _ in range(500):
            msg = SwitchView(view=ViewType.DASHBOARD)
            new_state, effects = reduce(state, msg)
            msg = SwitchView(view=ViewType.WORKTREE)
            state, effects = reduce(new_state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 1000) * 1000
        assert avg_ms < 1, f"SwitchView reduce took {avg_ms:.3f}ms (expected <1ms)"

    def test_reduce_key_navigation(self) -> None:
        """Test reduce with key navigation is fast."""
        commands = [
            Command(id=f"cmd_{i}", label=f"Command {i}")
            for i in range(10)
        ]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        start = time.perf_counter()
        for _ in range(500):
            # Navigate down
            msg = KeyPressed(key="j")
            state, effects = reduce(state, msg)
            # Navigate up
            msg = KeyPressed(key="k")
            state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 1000) * 1000
        assert avg_ms < 1, f"Key navigation reduce took {avg_ms:.3f}ms (expected <1ms)"

    def test_reduce_view_switching_keys(self) -> None:
        """Test reduce with view switching keys is fast."""
        state = AppState()

        start = time.perf_counter()
        for _ in range(500):
            # Switch to dashboard
            msg = KeyPressed(key="2")
            state, effects = reduce(state, msg)
            # Switch to worktree
            msg = KeyPressed(key="1")
            state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 1000) * 1000
        assert avg_ms < 1, f"View switching reduce took {avg_ms:.3f}ms (expected <1ms)"


class TestMemoryUsage:
    """Tests for memory efficiency."""

    def test_state_copy_efficiency(self) -> None:
        """Test that state copies are efficient."""
        commands = [
            Command(id=f"cmd_{i}", label=f"Command {i}", description=f"Description {i}")
            for i in range(100)
        ]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        start = time.perf_counter()
        for _ in range(100):
            # model_copy should be efficient for immutable state
            _ = state.model_copy(update={"running": not state.running})
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 100) * 1000
        assert avg_ms < 5, f"State copy took {avg_ms:.3f}ms (expected <5ms)"

    def test_no_state_mutation(self) -> None:
        """Test that reduce doesn't mutate original state."""
        commands = [
            Command(id="cmd_1", label="Command 1"),
            Command(id="cmd_2", label="Command 2"),
        ]
        worktree_view = WorktreeViewState(commands=commands)
        original_state = AppState(worktree_view=worktree_view)
        original_json = original_state.model_dump_json()

        # Run many reduces
        state = original_state
        for _ in range(100):
            msg = KeyPressed(key="j")
            state, effects = reduce(state, msg)
            msg = KeyPressed(key="k")
            state, effects = reduce(state, msg)

        # Original state should be unchanged
        assert original_state.model_dump_json() == original_json


class TestScalability:
    """Tests for scalability with larger data."""

    def test_large_command_list(self) -> None:
        """Test performance with large command list."""
        commands = [
            Command(id=f"cmd_{i}", label=f"Command {i}", description=f"Description {i}")
            for i in range(1000)
        ]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        # Serialization should still be reasonable
        start = time.perf_counter()
        _ = state.model_dump_json()
        elapsed = time.perf_counter() - start

        assert elapsed < 0.1, f"Serialization took {elapsed:.3f}s (expected <0.1s)"

        # Reduce should still be fast
        start = time.perf_counter()
        for _ in range(100):
            msg = KeyPressed(key="j")
            state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        avg_ms = (elapsed / 100) * 1000
        assert avg_ms < 5, f"Reduce with large list took {avg_ms:.3f}ms (expected <5ms)"

    def test_many_state_transitions(self) -> None:
        """Test many state transitions in sequence."""
        commands = [Command(id=f"cmd_{i}", label=f"Command {i}") for i in range(10)]
        worktree_view = WorktreeViewState(commands=commands)
        state = AppState(worktree_view=worktree_view)

        start = time.perf_counter()
        for _ in range(10000):
            msg = Tick()
            state, effects = reduce(state, msg)
        elapsed = time.perf_counter() - start

        throughput = 10000 / elapsed
        assert throughput > 10000, f"Throughput {throughput:.0f}/s (expected >10000/s)"
