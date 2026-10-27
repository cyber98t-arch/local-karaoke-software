import { fireEvent, render, screen } from "@testing-library/react";
import App from "./App";

describe("song library", () => {
  it("filters songs by Japanese title", () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText("曲名またはアーティストを検索"), { target: { value: "透明" } });
    expect(screen.getByText("透明な街")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /夜明けの航路/ })).not.toBeInTheDocument();
  });

  it("keeps the total score hidden during singing", () => {
    render(<App />);
    expect(screen.getByText(/歌唱中は総合点を表示しません/)).toBeInTheDocument();
    expect(screen.queryByText(/総合点: \d/)).not.toBeInTheDocument();
  });
});

