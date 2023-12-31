# Dang Language

### Roadmap

#### 16 Aug 2023

- [x] Redoing the entire lexer, now following the book "Writing an interpreter in GO" by Thorsten Ball
- [x] Start working on the parser and ASTLetStatement

#### 21 Aug 2023

- [x] Continue working on the parser.
- [x] Preparing the AST

#### 31 Aug 2023

- Parser.
- [x] Boolean
- [x] InfixExpressions
- [x] PrefixExpressions
- [x] Integer
- [x] Identifier
- [x] IfExpression
- [x] LetStatement
- [x] ReturnStatement
- [x] ExpressionStatement
- [x] BlockStatement
- [x] Continue working on the parser.

#### 01 Sept 2023

- Parser
- [x] FunctionLiteral
- [x] CallExpression
- [x] Clean up the parser

#### 03 Sept 2023

- [x] Finished the parser
- [x] Start working on the evaluator
- [x] Eval: Integer
- [x] Eval: Boolean
- [x] Eval: PrefixExpression

#### 04 Sept 2023

- [x] Eval: PrefixExpression
- [x] Eval: InfixExpression
- [x] Eval: IfExpression

#### 06 Sept 2023

- [x] Eval: IfExpression
- [x] Eval: ReturnStatement
- [ ] Eval: Error handling
- [x] Eval: LetStatement
- [x] Eval: Environment

#### 08 Sept 2023

- [x] Eval: FunctionLiteral
- [x] Eval: CallExpression

#### 08 Sept 2023

- [x] Token + Parse + Eval: Strings

#### 09 Sept 2023

- [x] Big refactor, removed traits and replaced them with enums
- [x] Built-in: 'len'

#### 10 Sept 2023

- [x] Token + Parse + Eval: Arrays
- [x] Token + Parse + Eval: Indexes

#### 13 Sept 2023

- [x] Built-in: 'first' + 'last'

#### 15 Sept 2023

- [x] HashMaps
- [x] Interpreter is now fully functional :D 🎉🎉🎉🎉🎉

#### 16 Sept 2023

- [x] Implemented basic error handling, no more panics
- [x] Implemented LTE and GTE
- [x] Implemented And and Or

#### 20 Sept 2023

- [x] Arrow functions.
- [x] Reassignment statements.

#### 05 Oct 2023

- [x] Fix: Closures
- [x] Fix: Return statmets, function body (block) must explicitly return a expression, if/else block implicitly return something (may change in the future)

#### 09 Oct 2023

- [x] Implemented While loops

#### 15 Oct 2023

- [x] Implemented (.) operation
