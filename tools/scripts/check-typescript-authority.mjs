import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

const root = process.cwd();

const authorityMutationNames = new Set([
  "accepted",
  "damage",
  "damageamount",
  "hitpoints",
  "currenthitpoints",
  "hp",
  "outcome",
  "rejection",
  "rejectioncode",
  "statefingerprint",
  "events",
  "trace",
]);

const authorityDerivationNames = new Set([
  "accepted",
  "damage",
  "damageamount",
  "hitpoints",
  "currenthitpoints",
  "hp",
  "outcome",
  "rejection",
  "rejectioncode",
  "statefingerprint",
]);

const semanticCallNames = new Set([
  "rolldie",
  "rolldice",
  "calculateattack",
  "calculatecheck",
  "calculatedamage",
  "derivedamage",
  "applydamage",
  "resolveattack",
  "resolvecheck",
  "resolvesavingthrow",
  "applymodifier",
  "mutategameplaystate",
]);

const computationOperators = new Set([
  ts.SyntaxKind.PlusToken,
  ts.SyntaxKind.MinusToken,
  ts.SyntaxKind.AsteriskToken,
  ts.SyntaxKind.SlashToken,
  ts.SyntaxKind.PercentToken,
  ts.SyntaxKind.AsteriskAsteriskToken,
  ts.SyntaxKind.GreaterThanToken,
  ts.SyntaxKind.GreaterThanEqualsToken,
  ts.SyntaxKind.LessThanToken,
  ts.SyntaxKind.LessThanEqualsToken,
  ts.SyntaxKind.EqualsEqualsToken,
  ts.SyntaxKind.EqualsEqualsEqualsToken,
  ts.SyntaxKind.ExclamationEqualsToken,
  ts.SyntaxKind.ExclamationEqualsEqualsToken,
  ts.SyntaxKind.AmpersandAmpersandToken,
  ts.SyntaxKind.BarBarToken,
]);

const assignmentOperators = new Set([
  ts.SyntaxKind.EqualsToken,
  ts.SyntaxKind.PlusEqualsToken,
  ts.SyntaxKind.MinusEqualsToken,
  ts.SyntaxKind.AsteriskEqualsToken,
  ts.SyntaxKind.SlashEqualsToken,
  ts.SyntaxKind.PercentEqualsToken,
]);

export function inspectTypeScriptAuthority(source, fileName = "fixture.ts") {
  const sourceFile = ts.createSourceFile(
    fileName,
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const diagnostics = [];
  const seen = new Set();

  const report = (node, message) => {
    const position = sourceFile.getLineAndCharacterOfPosition(
      node.getStart(sourceFile),
    );
    const diagnostic = `${fileName}:${position.line + 1}:${position.character + 1}: ${message}`;
    if (!seen.has(diagnostic)) {
      seen.add(diagnostic);
      diagnostics.push(diagnostic);
    }
  };

  const visit = (node) => {
    if (ts.isCallExpression(node)) {
      const callName = normalizedCallName(node.expression);
      if (isRandomCall(node.expression)) {
        report(
          node,
          "production TypeScript may not generate authority randomness",
        );
      } else if (semanticCallNames.has(callName)) {
        report(
          node,
          `production TypeScript may not execute rule semantics through ${node.expression.getText(sourceFile)}`,
        );
      }
    }

    if (ts.isVariableDeclaration(node) && node.initializer !== undefined) {
      const name = normalizedName(node.name);
      if (
        authorityDerivationNames.has(name) &&
        containsRuleComputation(node.initializer)
      ) {
        report(
          node,
          `production TypeScript may not derive authoritative ${node.name.getText(sourceFile)}`,
        );
      }
    }

    if (ts.isPropertyAssignment(node)) {
      const name = normalizedName(node.name);
      if (
        authorityDerivationNames.has(name) &&
        containsRuleComputation(node.initializer)
      ) {
        report(
          node,
          `production TypeScript may not compute authoritative ${node.name.getText(sourceFile)}`,
        );
      }
    }

    if (
      ts.isBinaryExpression(node) &&
      assignmentOperators.has(node.operatorToken.kind)
    ) {
      if (containsAuthorityName(node.left)) {
        report(
          node,
          `production TypeScript may not mutate authoritative state through ${node.left.getText(sourceFile)}`,
        );
      } else if (
        ts.isIdentifier(node.left) &&
        authorityDerivationNames.has(normalizedName(node.left)) &&
        containsRuleComputation(node.right)
      ) {
        report(
          node,
          `production TypeScript may not derive authoritative ${node.left.text}`,
        );
      }
    }

    if (
      (ts.isPrefixUnaryExpression(node) || ts.isPostfixUnaryExpression(node)) &&
      (node.operator === ts.SyntaxKind.PlusPlusToken ||
        node.operator === ts.SyntaxKind.MinusMinusToken) &&
      containsAuthorityName(node.operand)
    ) {
      report(
        node,
        `production TypeScript may not mutate authoritative state through ${node.operand.getText(sourceFile)}`,
      );
    }

    ts.forEachChild(node, visit);
  };

  visit(sourceFile);
  return diagnostics;
}

function containsRuleComputation(node) {
  let found = false;
  const visit = (candidate) => {
    if (found) return;
    if (
      ts.isBinaryExpression(candidate) &&
      computationOperators.has(candidate.operatorToken.kind) &&
      !isNullishComparison(candidate)
    ) {
      found = true;
      return;
    }
    if (ts.isCallExpression(candidate)) {
      const callName = normalizedCallName(candidate.expression);
      if (
        isRandomCall(candidate.expression) ||
        semanticCallNames.has(callName) ||
        isMathClamp(candidate.expression)
      ) {
        found = true;
        return;
      }
    }
    ts.forEachChild(candidate, visit);
  };
  visit(node);
  return found;
}

function containsAuthorityName(node) {
  if (ts.isIdentifier(node)) {
    return authorityMutationNames.has(normalizedName(node));
  }
  if (ts.isPropertyAccessExpression(node)) {
    return (
      authorityMutationNames.has(normalizedName(node.name)) ||
      containsAuthorityName(node.expression)
    );
  }
  if (ts.isElementAccessExpression(node)) {
    const argument = node.argumentExpression;
    return (
      containsAuthorityName(node.expression) ||
      (ts.isStringLiteral(argument) &&
        authorityMutationNames.has(normalizedName(argument)))
    );
  }
  return false;
}

function isRandomCall(expression) {
  const text = expression.getText();
  return text === "Math.random" || text === "crypto.getRandomValues";
}

function isMathClamp(expression) {
  const text = expression.getText();
  return text === "Math.min" || text === "Math.max";
}

function normalizedCallName(expression) {
  if (ts.isIdentifier(expression)) return normalizedName(expression);
  if (ts.isPropertyAccessExpression(expression)) {
    return normalizedName(expression.name);
  }
  return "";
}

function normalizedName(node) {
  if (ts.isIdentifier(node)) return node.text.toLowerCase();
  if (ts.isStringLiteral(node) || ts.isNumericLiteral(node)) {
    return node.text.toLowerCase();
  }
  return node
    .getText()
    .replace(/[^A-Za-z0-9]/g, "")
    .toLowerCase();
}

function isNullishComparison(node) {
  const equalityOperators = new Set([
    ts.SyntaxKind.EqualsEqualsToken,
    ts.SyntaxKind.EqualsEqualsEqualsToken,
    ts.SyntaxKind.ExclamationEqualsToken,
    ts.SyntaxKind.ExclamationEqualsEqualsToken,
  ]);
  if (!equalityOperators.has(node.operatorToken.kind)) return false;
  return isNullish(node.left) || isNullish(node.right);
}

function isNullish(node) {
  return (
    node.kind === ts.SyntaxKind.NullKeyword ||
    (ts.isIdentifier(node) && node.text === "undefined")
  );
}

function collectProductionTypeScript() {
  const files = [];
  for (const directory of [join(root, "apps"), join(root, "libs")]) {
    walk(directory, files);
  }
  return files.filter((file) => {
    const rel = relative(root, file).replaceAll("\\", "/");
    return (
      !rel.startsWith("apps/app-e2e/") &&
      !rel.startsWith("libs/testing-fixtures/") &&
      !rel.includes("/generated/") &&
      !rel.endsWith(".spec.ts") &&
      !rel.endsWith(".test.ts") &&
      !rel.endsWith(".d.ts")
    );
  });
}

function walk(directory, files) {
  for (const entry of readdirSync(directory)) {
    if (["node_modules", "dist", "coverage", ".git", ".nx"].includes(entry)) {
      continue;
    }
    const path = join(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) walk(path, files);
    else if (entry.endsWith(".ts")) files.push(path);
  }
}

function run() {
  const files = collectProductionTypeScript();
  const diagnostics = files.flatMap((file) =>
    inspectTypeScriptAuthority(
      readFileSync(file, "utf8"),
      relative(root, file).replaceAll("\\", "/"),
    ),
  );
  if (diagnostics.length > 0) {
    console.error(diagnostics.join("\n"));
    process.exit(1);
  }
  console.log(
    `check:typescript-authority ok (${files.length} production TypeScript files)`,
  );
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  run();
}
