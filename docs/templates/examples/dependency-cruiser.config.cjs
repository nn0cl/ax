// Example dependency-cruiser config. Copy to `<frontend-dir>/.dependency-cruiser.cjs`
// once the front-end project exists, and replace the forbidden paths below
// with your project's real boundary rules (e.g. "UI must not import the DB
// client" or "features must not import each other directly").

module.exports = {
  forbidden: [
    {
      name: "no-backend-transport-in-components",
      severity: "error",
      from: {
        path: "^src/(app|features|entities)/.*\\.(tsx|ts)$",
      },
      to: {
        path: "<replace-with-your-transport-sdk-import-path>",
      },
    },
    {
      name: "no-db-or-provider-sdk-in-ui",
      severity: "error",
      from: {
        path: "^src/",
      },
      to: {
        path: "<replace-with-db-or-provider-package-name-pattern>",
      },
    },
    {
      name: "no-circular",
      severity: "warn",
      from: {},
      to: {
        circular: true,
      },
    },
  ],
  options: {
    doNotFollow: {
      path: "node_modules",
    },
    tsPreCompilationDeps: true,
  },
};
