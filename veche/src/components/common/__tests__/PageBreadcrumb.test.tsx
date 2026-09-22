import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { PageBreadcrumb } from "../PageBreadCrumb";

describe("PageBreadcrumb component", () => {
  it("renders page title and home link", () => {
    render(
      <MemoryRouter>
        <PageBreadcrumb pageTitle="Users" />
      </MemoryRouter>,
    );

    expect(screen.getByText("Home")).toBeInTheDocument();
    expect(screen.getByText("Users")).toBeInTheDocument();
  });

  it("renders custom breadcrumb items", () => {
    render(
      <MemoryRouter>
        <PageBreadcrumb
          pageTitle="Edit User"
          items={[{ label: "Management", href: "/management" }]}
        />
      </MemoryRouter>,
    );

    expect(screen.getByText("Management")).toBeInTheDocument();
    expect(screen.getByText("Edit User")).toBeInTheDocument();
  });

  it("renders heading when showTitle is true", () => {
    render(
      <MemoryRouter>
        <PageBreadcrumb pageTitle="Dashboard" showTitle={true} />
      </MemoryRouter>,
    );

    const headings = screen.getAllByText("Dashboard");
    expect(headings.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByRole("heading", { level: 2, name: "Dashboard" })).toBeInTheDocument();
  });
});
