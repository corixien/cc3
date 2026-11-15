// Resolves `electron` to an empty module so apps/desktop sources that import it only for
// types or unused layers can load in plain Node (the real package downloads a binary on import).
export async function resolve(specifier, context, next) {
  if (specifier === "electron") {
    return { url: "data:text/javascript,export default {};", shortCircuit: true };
  }
  return next(specifier, context);
}
