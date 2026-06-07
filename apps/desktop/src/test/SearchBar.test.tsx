import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SearchBar } from "../components/SearchBar";

describe("SearchBar", () => {
  it("renders input with placeholder", () => {
    render(<SearchBar value="" onChange={() => {}} />);
    expect(screen.getByPlaceholderText("Search sessions, agents, repos...")).toBeTruthy();
  });

  it("displays the current value", () => {
    render(<SearchBar value="claude" onChange={() => {}} />);
    const input = screen.getByPlaceholderText("Search sessions, agents, repos...") as HTMLInputElement;
    expect(input.value).toBe("claude");
  });

  it("calls onChange when typing", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<SearchBar value="" onChange={onChange} />);
    const input = screen.getByPlaceholderText("Search sessions, agents, repos...");
    await user.type(input, "a");
    expect(onChange).toHaveBeenCalledWith("a");
  });

  it("shows clear button when value is not empty", () => {
    render(<SearchBar value="test" onChange={() => {}} />);
    expect(screen.getByRole("button")).toBeTruthy();
  });

  it("does not show clear button when value is empty", () => {
    render(<SearchBar value="" onChange={() => {}} />);
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("clears input when clear button is clicked", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<SearchBar value="test" onChange={onChange} />);
    await user.click(screen.getByRole("button"));
    expect(onChange).toHaveBeenCalledWith("");
  });
});
