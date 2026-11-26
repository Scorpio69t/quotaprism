import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  { ignores: ["src/generated/**", "dist/**"] },
  ...tseslint.configs.recommended,
  reactHooks.configs.flat.recommended,
);
