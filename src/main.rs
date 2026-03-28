use std::collections::HashMap;

// ============================================================================
// 原有的 AST 定义部分（全部保留）
// ============================================================================

#[derive(Debug, Clone)]//这个表示给接下来的这个结构（不论是数组还是结构体）赋予两个功能，一个是拷贝，一个是打印
//这两个功能实现由编译器自动完成
enum SExp {
    //首先，enum表示取值为这两种中的一种
    Atom(String),//一个原子
    List(Vec<SExp>),//vec列表，
}

#[derive(Debug, Clone)]
struct Program {
    commands: Vec<Command>,//commands:program这个结构体的内容，类型为一个Command向量
    //program最终承载parser的输出结果，也即parser最后实际输出的类型由Command承担
    //在struct中，：来表示类型
    
}

#[derive(Debug, Clone)]
enum Command {
    //由前可知，Command用来承载所有输出的指令结果
    //parser实际上完成的内容，就是翻译指令+指令下属的term

    // ---- 标准 SMT-LIB 前导命令 ----
    SetLogic(String),
    // (set-logic QF_ADT) => SetLogic("QF_ADT")
    // 声明使用的 logic，我们主要支持 QF_ADT（Quantifier-Free Algebraic Data Types）

    // ---- 声明类命令 ----
    DeclareDatatypes(Vec<DatatypeDecl>),
    //一个DeclareDatatypes，可能对应一次定义多个数据类型
    // 同时兼容 (declare-datatype T (...)) 单数形式，parser 会将其转换为此格式

    DeclareSort(String, usize),
    // (declare-sort Color 0) => DeclareSort("Color", 0)
    // 声明一个未解释 sort（仅注册名字和 arity）

    DeclareConst(String, SortExpr),
    // declare-const: 声明一个常量（0元函数）
    // 例如 (declare-const x List) => DeclareConst("x", Simple("List"))
    // 例如 (declare-const xs (List Int)) => DeclareConst("xs", Parametric("List",[Simple("Int")]))

    DeclareFun(String, Vec<SortExpr>, SortExpr),
    // declare-fun: 声明一个未解释函数
    // 例如 (declare-fun f (Int (List Int)) Bool)
    // 当参数列表为空时，等价于 declare-const

    // ---- 断言与求解 ----
    Assert(Formula),
    CheckSat,

    // ---- push/pop 断言栈 ----
    Push(usize),
    // (push N) — 保存当前断言栈状态 N 层，默认 N=1
    Pop(usize),
    // (pop N)  — 恢复断言栈状态 N 层，默认 N=1

    // ---- 模型查询 ----
    GetModel,
    // (get-model) — 在 check-sat 返回 sat 后，提取模型（各常量的值）
    GetValue(Vec<Term>),
    // (get-value (t1 t2 ...)) — 查询指定 term 在当前模型中的值

    // ---- 控制类命令 ----
    Exit,
    // (exit) — 标准 SMT-LIB 脚本终止命令

    Skip(String),
    // 已解析但语义上不影响求解的命令，String 为命令名
    // 涵盖: set-info, set-option, get-model, get-value, get-info,
    //        get-option, echo, push, pop, reset, reset-assertions 等
}





#[derive(Debug, Clone)]
struct DatatypeDecl {
    //对于每一个数据类型，信息如下
    name: String,//名字，比如list
    arity: usize,//参数数量，比如list 0，代表这个数据类型不需要任何参数。有的数据类型因为没有指定，需要参数来进行进一步指定
    type_params: Vec<String>,// par 参数名列表，例如 ["T"] 或 ["X","Y"]，arity=0 时为空
    constructors: Vec<ConstructorDecl>,//构造器，代表实际数据类型的结构，可能有多种选择，故也是一个Vec
}

#[derive(Debug, Clone)]
struct ConstructorDecl {
    //每一个构造器也可以有一个名字
    name: String,
    fields: Vec<FieldDecl>,//fields代表这个构造器的参数类型
}

#[derive(Debug, Clone)]
struct FieldDecl {
    selector: String,//selector就是parameter的名字，我们可以通过这个字符名去访问这个field
    sort: SortExpr,//就是type，Int什么的，SMT-LIB里叫sort。支持参数化 sort 如 (List T)
}

/// Sort 表达式：AST 层面的 sort 表示，支持简单和参数化 sort
///
/// 例如:
///   Int          => Simple("Int")
///   Bool         => Simple("Bool")
///   T            => Simple("T")   （在 par 上下文中是类型参数）
///   (List Int)   => Parametric("List", [Simple("Int")])
///   (Pair X Y)   => Parametric("Pair", [Simple("X"), Simple("Y")])
#[derive(Debug, Clone)]
enum SortExpr {
    Simple(String),
    Parametric(String, Vec<SortExpr>),
}

impl std::fmt::Display for SortExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortExpr::Simple(s) => write!(f, "{}", s),
            SortExpr::Parametric(name, args) => {
                write!(f, "({}", name)?;
                for a in args { write!(f, " {}", a)?; }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Formula {
    Eq(Term, Term),                                    // (= t1 t2)
    Not(Box<Formula>),                                 // (not φ)
    And(Vec<Formula>),                                 // (and φ1 φ2 ...)，至少 2 个子公式
    Or(Vec<Formula>),                                  // (or φ1 φ2 ...)，至少 2 个子公式
    Implies(Box<Formula>, Box<Formula>),               // (=> φ1 φ2)
    Ite(Box<Formula>, Box<Formula>, Box<Formula>),     // (ite φ_cond φ_then φ_else)，公式级 if-then-else
    IsTester(String, Term),                            // (is-Cons x) ADT tester，存储构造器名
    True,                                              // true
    False,                                             // false
}

#[derive(Debug, Clone)]
enum Term {
    //term有几种选项（可以用来比较等于/不等于的）
    Var(String),//字符型（单个）
    Int(i64),//整形（单个）
    App(String, Vec<Term>),//复杂term，一定是以某个操作符（比如head）开头的，然后后面可以嵌套。
    //我们这里还没有解释这些操作符到底有哪些，我们只是定义了其格式，正如前面我们也没有说明constructor具体可能有哪些一样，这些都在后面实现了

    // Term 级别的 ite（if-then-else）
    // 条件是 Formula（Bool sort），两个分支是 Term（任意 sort）
    // 例如 (ite (is-Nil x) 0 (head x)) => Ite(IsTester("Nil",x), Int(0), App("head",[x]))
    //
    // 为什么需要独立的 Term::Ite 而不是用 App("ite", ...) ？
    //   因为 ite 的条件是 Formula，分支是 Term，类型不同。
    //   如果用 App 处理，条件会被 parse_term 解析为普通 term，
    //   丢失 Formula 结构（如 IsTester、And、Or 等），无法正确编码到 egg。
    Ite(Box<Formula>, Box<Term>, Box<Term>),

    // match 表达式（ADT 模式匹配）
    // (match scrutinee (pattern1 body1) (pattern2 body2) ...)
    // 在 egg 编码时脱糖为嵌套的 ite + tester + selector：
    //   (match x (Nil 0) ((Cons h t) h))
    //   => (ite (is-Nil x) 0 (ite (is-Cons x) (head x) ???))
    // 最后一个 case 作为 else 分支直接使用（无需 tester 条件）
    Match(Box<Term>, Vec<MatchCase>),

    // (as term sort) — 类型消歧（Sort Qualification）
    // 用于参数化数据类型中，为多态构造器指定具体 sort 实例
    // 例如 (as nil (List Int)) => As(Var("nil"), Parametric("List",[Simple("Int")]))
    // 在 egg 编码时执行 sort 擦除（直接编码内部 term），sort 信息仅用于类型检查
    As(Box<Term>, SortExpr),
}

/// match 表达式中的一个分支
#[derive(Debug, Clone)]
struct MatchCase {
    pattern: MatchPattern,
    body: Term,
}

/// match 模式
#[derive(Debug, Clone)]
enum MatchPattern {
    /// (Ctor v1 v2 ...) — 构造器模式，绑定变量到各 field
    /// 例如 (Cons h t) => Constructor("Cons", ["h", "t"])
    Constructor(String, Vec<String>),
    /// 原子模式 — 可能是无参构造器（如 Nil）或绑定变量（catch-all）
    /// 在 egg 编码时根据 ctor_to_dt 映射来区分
    Atom(String),
}


// ============================================================================
// Sort 表示与类型检查
// ============================================================================
//
// SMT-LIB 中每个 term 都有一个 sort（类型）。在 QF_ADT 逻辑中，sort 包括：
//   - Bool: 布尔类型（true/false 以及所有公式的 sort）
//   - Int:  整数类型
//   - 用户定义的 ADT sort（如 List、Tree 等，通过 declare-datatypes 引入）
//
// Sort 系统的作用：
//   1. 验证 (=) 两侧的 term 具有相同的 sort
//   2. 验证构造器/selector/函数的参数 sort 正确
//   3. 验证 ite 的两个分支 sort 一致
//   4. 验证 tester 的参数 sort 是对应的 ADT
//   5. 验证 match 的 pattern 与 scrutinee sort 一致，所有 case body sort 相同
//
// 类型检查作为 egg 编码之前的验证层，不修改 AST 或 egg 编码逻辑。

#[derive(Debug, Clone, PartialEq, Eq)]
enum Sort {
    Bool,
    Int,
    Named(String),           // arity-0 用户 sort (如 Color)
    Param(String),           // 类型参数 (par 上下文中的 T, X, Y)
    App(String, Vec<Sort>),  // 参数化 sort 实例: (List Int), (Pair Bool Int)
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::Bool => write!(f, "Bool"),
            Sort::Int => write!(f, "Int"),
            Sort::Named(s) | Sort::Param(s) => write!(f, "{}", s),
            Sort::App(name, args) => {
                write!(f, "({}", name)?;
                for a in args { write!(f, " {}", a)?; }
                write!(f, ")")
            }
        }
    }
}

/// 简单 sort 名 -> Sort（仅处理原子名，不处理参数化）
fn parse_sort_name(name: &str) -> Sort {
    match name {
        "Bool" => Sort::Bool,
        "Int" => Sort::Int,
        _ => Sort::Named(name.to_string()),
    }
}

/// SortExpr -> Sort 转换，type_params 中的名称映射为 Sort::Param
fn sort_expr_to_sort(expr: &SortExpr, type_params: &[String]) -> Sort {
    match expr {
        SortExpr::Simple(name) => {
            if type_params.contains(name) {
                Sort::Param(name.clone())
            } else {
                parse_sort_name(name)
            }
        }
        SortExpr::Parametric(name, args) => {
            Sort::App(name.clone(), args.iter().map(|a| sort_expr_to_sort(a, type_params)).collect())
        }
    }
}

/// Sort 统一（unification）：将 pattern 中的 Param 与 concrete sort 匹配
fn unify_sorts(pattern: &Sort, concrete: &Sort, bindings: &mut HashMap<String, Sort>) -> Result<(), String> {
    match (pattern, concrete) {
        (Sort::Param(name), _) => {
            if let Some(existing) = bindings.get(name) {
                if existing != concrete {
                    return Err(format!("类型参数 '{}' 绑定冲突: {} vs {}", name, existing, concrete));
                }
            } else {
                bindings.insert(name.clone(), concrete.clone());
            }
            Ok(())
        }
        (Sort::Bool, Sort::Bool) | (Sort::Int, Sort::Int) => Ok(()),
        (Sort::Named(a), Sort::Named(b)) if a == b => Ok(()),
        (Sort::App(a, args_a), Sort::App(b, args_b)) if a == b && args_a.len() == args_b.len() => {
            for (pa, pb) in args_a.iter().zip(args_b) {
                unify_sorts(pa, pb, bindings)?;
            }
            Ok(())
        }
        _ => Err(format!("sort 不匹配: {} vs {}", pattern, concrete)),
    }
}

/// 对 Sort 应用类型参数绑定
fn substitute_sort(sort: &Sort, bindings: &HashMap<String, Sort>) -> Sort {
    match sort {
        Sort::Param(name) => bindings.get(name).cloned().unwrap_or_else(|| sort.clone()),
        Sort::App(name, args) => Sort::App(
            name.clone(), args.iter().map(|a| substitute_sort(a, bindings)).collect(),
        ),
        _ => sort.clone(),
    }
}

/// 构造器签名（支持多态）
#[derive(Debug, Clone)]
struct CtorSig {
    type_params: Vec<String>,  // 空 = 单态
    field_sorts: Vec<Sort>,    // 可能含 Param
    result_sort: Sort,         // 可能含 Param
}

/// Selector 签名（支持多态）
#[derive(Debug, Clone)]
struct SelectorSig {
    type_params: Vec<String>,
    input_sort: Sort,
    output_sort: Sort,
}

/// Sort 环境：收集所有声明中的类型信息，支持参数化数据类型和 as 消歧
#[derive(Debug, Clone)]
struct SortEnv {
    known_sorts: Vec<String>,
    sort_arities: HashMap<String, usize>,
    const_sorts: HashMap<String, Sort>,
    fun_sigs: HashMap<String, (Vec<Sort>, Sort)>,
    ctor_sigs: HashMap<String, CtorSig>,
    selector_sigs: HashMap<String, SelectorSig>,
}

impl SortEnv {
    fn new() -> Self {
        let mut env = SortEnv {
            known_sorts: vec!["Bool".to_string(), "Int".to_string()],
            sort_arities: HashMap::new(),
            const_sorts: HashMap::new(),
            fun_sigs: HashMap::new(),
            ctor_sigs: HashMap::new(),
            selector_sigs: HashMap::new(),
        };
        env.sort_arities.insert("Bool".to_string(), 0);
        env.sort_arities.insert("Int".to_string(), 0);
        env.const_sorts.insert("true".to_string(), Sort::Bool);
        env.const_sorts.insert("false".to_string(), Sort::Bool);
        env
    }

    fn register_sort(&mut self, name: &str) {
        if !self.known_sorts.contains(&name.to_string()) {
            self.known_sorts.push(name.to_string());
        }
    }

    fn register_sort_with_arity(&mut self, name: &str, arity: usize) {
        self.register_sort(name);
        self.sort_arities.insert(name.to_string(), arity);
    }

    /// 检查 SortExpr 的合法性（sort 已知 + arity 匹配）
    fn check_sort_expr(&self, expr: &SortExpr, type_params: &[String]) -> Result<(), String> {
        match expr {
            SortExpr::Simple(name) => {
                if type_params.contains(name) { return Ok(()); }
                if !self.known_sorts.contains(name) {
                    return Err(format!("类型错误: 未知的 sort '{}'", name));
                }
                if let Some(&arity) = self.sort_arities.get(name) {
                    if arity > 0 {
                        return Err(format!("类型错误: sort '{}' 需要 {} 个类型参数", name, arity));
                    }
                }
                Ok(())
            }
            SortExpr::Parametric(name, args) => {
                if !self.known_sorts.contains(name) {
                    return Err(format!("类型错误: 未知的 sort '{}'", name));
                }
                if let Some(&arity) = self.sort_arities.get(name) {
                    if args.len() != arity {
                        return Err(format!("类型错误: sort '{}' 期望 {} 个类型参数, 实际 {}",
                            name, arity, args.len()));
                    }
                }
                for a in args { self.check_sort_expr(a, type_params)?; }
                Ok(())
            }
        }
    }

    fn register_datatype(&mut self, dt: &DatatypeDecl) {
        self.register_sort_with_arity(&dt.name, dt.arity);

        // 结果 sort：arity=0 用 Named，arity>0 用 App(name, [Param...])
        let result_sort = if dt.type_params.is_empty() {
            Sort::Named(dt.name.clone())
        } else {
            Sort::App(dt.name.clone(),
                dt.type_params.iter().map(|p| Sort::Param(p.clone())).collect())
        };

        for ctor in &dt.constructors {
            let field_sorts: Vec<Sort> = ctor.fields.iter()
                .map(|f| sort_expr_to_sort(&f.sort, &dt.type_params))
                .collect();
            self.ctor_sigs.insert(ctor.name.clone(), CtorSig {
                type_params: dt.type_params.clone(),
                field_sorts: field_sorts.clone(),
                result_sort: result_sort.clone(),
            });
            for (field, fs) in ctor.fields.iter().zip(&field_sorts) {
                self.selector_sigs.insert(field.selector.clone(), SelectorSig {
                    type_params: dt.type_params.clone(),
                    input_sort: result_sort.clone(),
                    output_sort: fs.clone(),
                });
            }
        }
    }

    fn register_const(&mut self, name: &str, sort: Sort) {
        self.const_sorts.insert(name.to_string(), sort);
    }
    fn register_fun(&mut self, name: &str, arg_sorts: Vec<Sort>, ret_sort: Sort) {
        self.fun_sigs.insert(name.to_string(), (arg_sorts, ret_sort));
    }

    // ---- 类型推断与检查 ----

    fn infer_term_sort(&self, term: &Term, local_scope: &HashMap<String, Sort>) -> Result<Sort, String> {
        match term {
            Term::Var(name) => {
                if let Some(sort) = local_scope.get(name) { return Ok(sort.clone()); }
                if let Some(sort) = self.const_sorts.get(name) { return Ok(sort.clone()); }
                if let Some(sig) = self.ctor_sigs.get(name) {
                    if sig.field_sorts.is_empty() {
                        // 无参构造器: 单态直接返回; 多态需要 as 消歧
                        if sig.type_params.is_empty() {
                            return Ok(sig.result_sort.clone());
                        } else {
                            return Err(format!(
                                "类型错误: 多态构造器 '{}' 需要 (as {} <sort>) 消歧", name, name));
                        }
                    } else {
                        return Err(format!("类型错误: 构造器 '{}' 需要 {} 个参数",
                            name, sig.field_sorts.len()));
                    }
                }
                Err(format!("类型错误: 未声明的符号 '{}'", name))
            }

            Term::Int(_) => Ok(Sort::Int),

            Term::App(func, args) => {
                let arg_sorts: Vec<Sort> = args.iter()
                    .map(|a| self.infer_term_sort(a, local_scope))
                    .collect::<Result<_, _>>()?;
                self.infer_app_sort(func, &arg_sorts)
            }

            Term::Ite(cond, then_t, else_t) => {
                self.check_formula(cond, local_scope)?;
                let ts = self.infer_term_sort(then_t, local_scope)?;
                let es = self.infer_term_sort(else_t, local_scope)?;
                if ts != es {
                    return Err(format!("类型错误: ite 两分支 sort 不一致: {}  vs {}", ts, es));
                }
                Ok(ts)
            }

            Term::Match(scrut, cases) => {
                let scrut_sort = self.infer_term_sort(scrut, local_scope)?;
                if cases.is_empty() { return Err("类型错误: match 至少需要一个 case".into()); }
                let mut result_sort: Option<Sort> = None;
                for case in cases {
                    let cs = self.infer_match_case_sort(&scrut_sort, case, local_scope)?;
                    if let Some(ref rs) = result_sort {
                        if &cs != rs {
                            return Err(format!("类型错误: match case sort 不一致: {} vs {}", rs, cs));
                        }
                    } else { result_sort = Some(cs); }
                }
                Ok(result_sort.unwrap())
            }

            Term::As(inner, sort_expr) => {
                let target = sort_expr_to_sort(sort_expr, &[]);
                self.infer_as_sort(inner, &target, local_scope)
            }
        }
    }

    /// 推断 (as term sort) 的 sort
    fn infer_as_sort(&self, inner: &Term, target: &Sort, local_scope: &HashMap<String, Sort>) -> Result<Sort, String> {
        match inner {
            Term::Var(name) => {
                // 无参构造器 + as 消歧
                if let Some(sig) = self.ctor_sigs.get(name.as_str()) {
                    if !sig.field_sorts.is_empty() {
                        return Err(format!("类型错误: 构造器 '{}' 期望 {} 个参数", name, sig.field_sorts.len()));
                    }
                    let mut bindings = HashMap::new();
                    unify_sorts(&sig.result_sort, target, &mut bindings)
                        .map_err(|e| format!("类型错误: (as {} {}) — {}", name, target, e))?;
                    return Ok(target.clone());
                }
                // 非构造器: 推断内部 sort 并检查一致
                let inner_sort = self.infer_term_sort(inner, local_scope)?;
                if &inner_sort != target {
                    return Err(format!("类型错误: (as ...) 标注 {} 与推断 {} 不一致", target, inner_sort));
                }
                Ok(target.clone())
            }
            Term::App(func, args) => {
                let arg_sorts: Vec<Sort> = args.iter()
                    .map(|a| self.infer_term_sort(a, local_scope))
                    .collect::<Result<_, _>>()?;
                // 构造器 + as: 用 target 驱动 unification
                if let Some(sig) = self.ctor_sigs.get(func.as_str()) {
                    if arg_sorts.len() != sig.field_sorts.len() {
                        return Err(format!("类型错误: 构造器 '{}' 期望 {} 个参数, 实际 {}",
                            func, sig.field_sorts.len(), arg_sorts.len()));
                    }
                    let mut bindings = HashMap::new();
                    unify_sorts(&sig.result_sort, target, &mut bindings)
                        .map_err(|e| format!("类型错误: (as {} {}) — {}", func, target, e))?;
                    for (i, (actual, expected)) in arg_sorts.iter().zip(&sig.field_sorts).enumerate() {
                        let exp_inst = substitute_sort(expected, &bindings);
                        if actual != &exp_inst {
                            return Err(format!("类型错误: 构造器 '{}' 第 {} 个参数: 期望 {}, 实际 {}",
                                func, i+1, exp_inst, actual));
                        }
                    }
                    return Ok(target.clone());
                }
                // 非构造器 fallback
                let inner_sort = self.infer_app_sort(func.as_str(), &arg_sorts)?;
                if &inner_sort != target {
                    return Err(format!("类型错误: (as ...) 标注 {} 与推断 {} 不一致", target, inner_sort));
                }
                Ok(target.clone())
            }
            _ => {
                let inner_sort = self.infer_term_sort(inner, local_scope)?;
                if &inner_sort != target {
                    return Err(format!("类型错误: (as ...) 标注 {} 与推断 {} 不一致", target, inner_sort));
                }
                Ok(target.clone())
            }
        }
    }

    /// 推断函数/构造器应用的 sort（统一处理 unification）
    fn infer_app_sort(&self, func: &str, arg_sorts: &[Sort]) -> Result<Sort, String> {
        // 1. 构造器
        if let Some(sig) = self.ctor_sigs.get(func) {
            if arg_sorts.len() != sig.field_sorts.len() {
                return Err(format!("类型错误: 构造器 '{}' 期望 {} 个参数, 实际 {}",
                    func, sig.field_sorts.len(), arg_sorts.len()));
            }
            let mut bindings = HashMap::new();
            for (i, (actual, expected)) in arg_sorts.iter().zip(&sig.field_sorts).enumerate() {
                unify_sorts(expected, actual, &mut bindings)
                    .map_err(|_| format!("类型错误: 构造器 '{}' 第 {} 个参数: 期望 sort {}, 实际 sort {}",
                        func, i+1, substitute_sort(expected, &bindings), actual))?;
            }
            return Ok(substitute_sort(&sig.result_sort, &bindings));
        }
        // 2. Selector
        if let Some(sig) = self.selector_sigs.get(func) {
            if arg_sorts.len() != 1 {
                return Err(format!("类型错误: selector '{}' 期望 1 个参数, 实际 {}", func, arg_sorts.len()));
            }
            let mut bindings = HashMap::new();
            unify_sorts(&sig.input_sort, &arg_sorts[0], &mut bindings)
                .map_err(|_| format!("类型错误: selector '{}' 期望参数 sort {}, 实际 {}",
                    func, sig.input_sort, arg_sorts[0]))?;
            return Ok(substitute_sort(&sig.output_sort, &bindings));
        }
        // 3. 声明函数
        if let Some((expected, ret)) = self.fun_sigs.get(func) {
            if arg_sorts.len() != expected.len() {
                return Err(format!("类型错误: 函数 '{}' 期望 {} 个参数, 实际 {}",
                    func, expected.len(), arg_sorts.len()));
            }
            for (i, (actual, exp)) in arg_sorts.iter().zip(expected).enumerate() {
                if actual != exp {
                    return Err(format!("类型错误: 函数 '{}' 第 {} 个参数: 期望 {}, 实际 {}",
                        func, i+1, exp, actual));
                }
            }
            return Ok(ret.clone());
        }
        Err(format!("类型错误: 未声明的函数/构造器 '{}'", func))
    }

    fn infer_match_case_sort(&self, scrut_sort: &Sort, case: &MatchCase, outer_scope: &HashMap<String, Sort>) -> Result<Sort, String> {
        let mut local_scope = outer_scope.clone();
        match &case.pattern {
            MatchPattern::Constructor(ctor_name, vars) => {
                if let Some(sig) = self.ctor_sigs.get(ctor_name) {
                    // unify result_sort with scrut_sort to get param bindings
                    let mut bindings = HashMap::new();
                    unify_sorts(&sig.result_sort, scrut_sort, &mut bindings)
                        .map_err(|_| format!("类型错误: match 构造器 '{}' sort {} 与 scrutinee sort {} 不匹配",
                            ctor_name, sig.result_sort, scrut_sort))?;
                    if vars.len() != sig.field_sorts.len() {
                        return Err(format!("类型错误: 构造器 '{}' pattern 变量数 ({}) 与 field 数 ({}) 不匹配",
                            ctor_name, vars.len(), sig.field_sorts.len()));
                    }
                    for (var, fs) in vars.iter().zip(&sig.field_sorts) {
                        local_scope.insert(var.clone(), substitute_sort(fs, &bindings));
                    }
                } else {
                    return Err(format!("类型错误: match 中未知构造器 '{}'", ctor_name));
                }
            }
            MatchPattern::Atom(name) => {
                if let Some(sig) = self.ctor_sigs.get(name) {
                    let mut bindings = HashMap::new();
                    unify_sorts(&sig.result_sort, scrut_sort, &mut bindings).map_err(|_|
                        format!("类型错误: match pattern '{}' sort {} 与 scrutinee {} 不匹配",
                            name, sig.result_sort, scrut_sort))?;
                } else {
                    local_scope.insert(name.clone(), scrut_sort.clone());
                }
            }
        }
        self.infer_term_sort(&case.body, &local_scope)
    }

    fn check_formula(&self, formula: &Formula, local_scope: &HashMap<String, Sort>) -> Result<(), String> {
        match formula {
            Formula::True | Formula::False => Ok(()),
            Formula::Eq(t1, t2) => {
                let s1 = self.infer_term_sort(t1, local_scope)?;
                let s2 = self.infer_term_sort(t2, local_scope)?;
                if s1 != s2 {
                    return Err(format!("类型错误: '=' 两侧 sort 不一致: {} vs {}", s1, s2));
                }
                Ok(())
            }
            Formula::Not(f) => self.check_formula(f, local_scope),
            Formula::And(fs) | Formula::Or(fs) => { for f in fs { self.check_formula(f, local_scope)?; } Ok(()) }
            Formula::Implies(l, r) => { self.check_formula(l, local_scope)?; self.check_formula(r, local_scope) }
            Formula::Ite(c, t, e) => { self.check_formula(c, local_scope)?; self.check_formula(t, local_scope)?; self.check_formula(e, local_scope) }
            Formula::IsTester(ctor_name, term) => {
                let ts = self.infer_term_sort(term, local_scope)?;
                if let Some(sig) = self.ctor_sigs.get(ctor_name) {
                    let mut bindings = HashMap::new();
                    unify_sorts(&sig.result_sort, &ts, &mut bindings).map_err(|_|
                        format!("类型错误: (is-{}) 期望 ADT sort {}, 实际 {}", ctor_name, sig.result_sort, ts))?;
                    Ok(())
                } else { Err(format!("类型错误: 未知构造器 '{}'", ctor_name)) }
            }
        }
    }
}


// ============================================================================
// 原有的 Tokenizer & S-Expression Parser 部分（全部保留）
// ============================================================================

fn tokenize(input: &str) -> Vec<String> {
    //做tokenize的函数，tokenize的方法是去掉空格，保留所有的字符和中括号
    //
    // 新增支持：
    //   - ; 行注释：从 ; 到行尾的内容被忽略
    //   - "..." 字符串字面量：整体作为一个 token（含引号），内部空格不分割
    //   - |...| 引用符号：SMT-LIB 中 |foo bar| 是一个合法的符号名
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_comment = false;   // 正在跳过行注释
    let mut in_string = false;    // 正在收集字符串字面量
    let mut in_quoted = false;    // 正在收集 |...| 引用符号

    for ch in input.chars() {
        // ---- 行注释模式：跳到行尾 ----
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        // ---- 字符串字面量模式：收集到闭合引号 ----
        if in_string {
            cur.push(ch);
            if ch == '"' {
                tokens.push(cur.clone());
                cur.clear();
                in_string = false;
            }
            continue;
        }

        // ---- 引用符号模式：收集到闭合 | ----
        if in_quoted {
            cur.push(ch);
            if ch == '|' {
                tokens.push(cur.clone());
                cur.clear();
                in_quoted = false;
            }
            continue;
        }

        // ---- 正常模式 ----
        match ch {
            ';' => {
                // 行注释开始：先刷出当前 token
                if !cur.trim().is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
                in_comment = true;
            }
            '"' => {
                // 字符串字面量开始
                if !cur.trim().is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
                cur.push('"');
                in_string = true;
            }
            '|' => {
                // 引用符号开始
                if !cur.trim().is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
                cur.push('|');
                in_quoted = true;
            }
            '(' | ')' => {
                if !cur.trim().is_empty() {
                    tokens.push(cur.clone());
                }
                cur.clear();
                tokens.push(ch.to_string());
            }
            c if c.is_whitespace() => {
                if !cur.trim().is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(ch),
        }
    }

    if !cur.trim().is_empty() {
        tokens.push(cur);
    }

    tokens
}


fn parse_sexp(tokens: &[String], pos: &mut usize) -> Result<SExp, String> {
    //将个字符串tokens向量，转化为以SExp（通过括号，分割为Atom，list的格式）

    if *pos >= tokens.len() {
        return Err("unexpected end of input".into());
    }

    match tokens[*pos].as_str() {
        "(" => {
            *pos += 1;
            let mut items = Vec::new();
            while *pos < tokens.len() && tokens[*pos] != ")" {
                items.push(parse_sexp(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err("missing ')'".into());
            }
            *pos += 1;
            Ok(SExp::List(items))
        }
        ")" => Err("unexpected ')'".into()),
        atom => {
            *pos += 1;
            Ok(SExp::Atom(atom.to_string()))
        }
    }
}



fn parse_all_sexps(input: &str) -> Result<Vec<SExp>, String> {
    //一次parse完所有的表达式
    let tokens = tokenize(input);
    let mut pos = 0;
    let mut res = Vec::new();
    while pos < tokens.len() {
        res.push(parse_sexp(&tokens, &mut pos)?);
    }
    Ok(res)
}



fn parse_program(input: &str) -> Result<Program, String> {
    //总的调用tokenize，再parse all，完成所有字符串向SExp的转化。
    let sexps = parse_all_sexps(input)?;
    let mut commands = Vec::new();

    for sexp in sexps {
        commands.push(parse_command(&sexp)?);
    }

    Ok(Program { commands })
}

/*
以上是转化为SExp的过程，接下来，我们将针对我们的SExp，提取各个部分的语义，然后将其转化为各部分语义完成归类的AST
*/


fn parse_command(sexp: &SExp) -> Result<Command, String> {
    // 所有 command 的解析入口
    // 输入 SExp，返回 Command 类型
    // 支持标准 SMT-LIB 2.6 的所有常见命令
    match sexp {
        SExp::List(items) if !items.is_empty() => {
            let cmd_name = match &items[0] {
                SExp::Atom(s) => s.as_str(),
                _ => return Err(format!("expected command name, got {:?}", items[0])),
            };

            match cmd_name {
                // ---- 标准前导命令 ----
                "set-logic" => {
                    // (set-logic QF_ADT)
                    if items.len() != 2 {
                        return Err("set-logic expects 1 argument".into());
                    }
                    Ok(Command::SetLogic(atom(&items[1])?.to_string()))
                }

                // ---- 声明类命令 ----
                "declare-datatypes" => {
                    // (declare-datatypes ((List 0)) (((Nil) (Cons ...))))
                    // 复数形式：一次声明多个数据类型
                    parse_declare_datatypes(items)
                }
                "declare-datatype" => {
                    // (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
                    // 单数形式：一次声明一个数据类型，arity 默认为 0
                    // 将其转换为复数形式的 DeclareDatatypes
                    parse_declare_datatype(items)
                }
                "declare-sort" => {
                    // (declare-sort Color 0)
                    if items.len() != 3 {
                        return Err("declare-sort expects 2 arguments: name and arity".into());
                    }
                    let name = atom(&items[1])?.to_string();
                    let arity = atom(&items[2])?
                        .parse::<usize>()
                        .map_err(|_| "invalid sort arity".to_string())?;
                    Ok(Command::DeclareSort(name, arity))
                }
                "declare-const" => {
                    parse_declare_const(items)
                }
                "declare-fun" => {
                    parse_declare_fun(items)
                }

                // ---- 断言与求解 ----
                "assert" => {
                    if items.len() != 2 {
                        return Err("assert expects 1 argument".into());
                    }
                    Ok(Command::Assert(parse_formula(&items[1])?))
                }
                "check-sat" => Ok(Command::CheckSat),

                // ---- 控制命令 ----
                "exit" => Ok(Command::Exit),

                // ---- push/pop 断言栈 ----
                "push" => {
                    // (push) 或 (push N)，默认 N=1
                    let n = if items.len() > 1 {
                        atom(&items[1])?.parse::<usize>()
                            .map_err(|_| "push: invalid numeric argument".to_string())?
                    } else { 1 };
                    Ok(Command::Push(n))
                }
                "pop" => {
                    // (pop) 或 (pop N)，默认 N=1
                    let n = if items.len() > 1 {
                        atom(&items[1])?.parse::<usize>()
                            .map_err(|_| "pop: invalid numeric argument".to_string())?
                    } else { 1 };
                    Ok(Command::Pop(n))
                }

                // ---- 模型查询 ----
                "get-model" => Ok(Command::GetModel),
                "get-value" => {
                    // (get-value (t1 t2 ...))
                    if items.len() != 2 {
                        return Err("get-value expects 1 argument: a list of terms".into());
                    }
                    match &items[1] {
                        SExp::List(term_sexps) => {
                            let terms: Result<Vec<Term>, String> =
                                term_sexps.iter().map(|s| parse_term(s)).collect();
                            Ok(Command::GetValue(terms?))
                        }
                        _ => Err("get-value: expected a list of terms".into()),
                    }
                }

                // ---- 语义无关的命令：解析但跳过 ----
                // 这些命令在标准 SMT-LIB 文件中常见，但不影响我们的等式推理
                "set-info"
                | "set-option"
                | "get-info"
                | "get-option"
                | "get-unsat-core"
                | "get-proof"
                | "get-assertions"
                | "get-assignment"
                | "echo"
                | "reset"
                | "reset-assertions"
                | "define-sort"
                | "define-fun"
                | "define-fun-rec"
                | "define-funs-rec"
                | "check-sat-assuming" => {
                    Ok(Command::Skip(cmd_name.to_string()))
                }

                _ => {
                    // 未识别的命令：发出警告但不报错，保持对未知扩展的鲁棒性
                    eprintln!("[警告] 未识别的命令 '{}', 将被跳过", cmd_name);
                    Ok(Command::Skip(cmd_name.to_string()))
                }
            }
        },
        _ => Err(format!("expected command, got {:?}", sexp)),
    }
}


fn parse_declare_datatypes(items: &[SExp]) -> Result<Command, String> {
    //当解析当前command是declare-datetypes时
    if items.len() != 3 {
        return Err("declare-datatypes expects 2 arguments".into());
        //继续格式检查，declare datatype应该有三个参数，类型名字，arity，constructor
        //注意这里不包含命令名字？反正最后command里是不包含命令名字的
    }

    let sort_decls = match &items[1] {
        SExp::List(xs) => xs,
        _ => return Err("expected sort declaration list".into()),
    };

    let datatype_bodies = match &items[2] {
        SExp::List(xs) => xs,
        _ => return Err("expected datatype body list".into()),
    };

    if sort_decls.len() != datatype_bodies.len() {
        return Err("sort decl count != datatype body count".into());
    }

    let mut dts = Vec::new();

    for (sd, body) in sort_decls.iter().zip(datatype_bodies.iter()) {
        let (name, arity) = parse_sort_decl(sd)?;
        // 解析 body: 可能是 (par (T ...) ((ctors...))) 或直接 ((ctors...))
        let (type_params, constructors) = parse_datatype_body(body)?;
        if !type_params.is_empty() && type_params.len() != arity {
            return Err(format!(
                "par 参数数量 ({}) 与声明的 arity ({}) 不匹配: {}",
                type_params.len(), arity, name));
        }
        dts.push(DatatypeDecl { name, arity, type_params, constructors });
    }

    Ok(Command::DeclareDatatypes(dts))
}

/// 解析数据类型 body: 可能是 (par (T ...) ((ctors...))) 或直接 ((ctors...))
///
/// par 形式:
///   (par (T) ((nil) (cons (hd T) (tl (List T)))))
///   => type_params = ["T"], constructors = [nil, cons(...)]
///
/// 普通形式:
///   ((Nil) (Cons (head Int) (tail List)))
///   => type_params = [], constructors = [Nil, Cons(...)]
fn parse_datatype_body(sexp: &SExp) -> Result<(Vec<String>, Vec<ConstructorDecl>), String> {
    match sexp {
        SExp::List(xs) if !xs.is_empty() => {
            // 检查第一个元素是否是 "par"
            if let SExp::Atom(s) = &xs[0] {
                if s == "par" {
                    if xs.len() != 3 {
                        return Err("par expects 2 arguments: type params and constructor list".into());
                    }
                    let params = match &xs[1] {
                        SExp::List(ps) => {
                            ps.iter().map(|p| atom(p).map(|s| s.to_string()))
                                .collect::<Result<Vec<_>, _>>()?
                        }
                        _ => return Err("par: expected list of type parameters".into()),
                    };
                    let constructors = parse_constructors(&xs[2])?;
                    return Ok((params, constructors));
                }
            }
            // 无 par 包装: 直接解析构造器列表
            let constructors = parse_constructors(sexp)?;
            Ok((vec![], constructors))
        }
        _ => Err("invalid datatype body".into()),
    }
}

/// 解析 (declare-datatype <name> (<constructor_decl>+))
/// 这是 SMT-LIB 的单数形式，等价于 (declare-datatypes ((<name> 0)) ((<ctors>)))
/// 例如:
///   (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
///   等价于:
///   (declare-datatypes ((List 0)) (((Nil) (Cons (head Int) (tail List)))))
fn parse_declare_datatype(items: &[SExp]) -> Result<Command, String> {
    if items.len() != 3 {
        return Err("declare-datatype expects 2 arguments: name and constructor list".into());
    }
    let name = atom(&items[1])?.to_string();
    let (type_params, constructors) = parse_datatype_body(&items[2])?;
    let arity = type_params.len();
    Ok(Command::DeclareDatatypes(vec![DatatypeDecl {
        name,
        arity,
        type_params,
        constructors,
    }]))
}

fn parse_sort_decl(sexp: &SExp) -> Result<(String, usize), String> {
    match sexp {
        SExp::List(xs) if xs.len() == 2 => {
            let name = atom(&xs[0])?.to_string();
            let arity = atom(&xs[1])?
                .parse::<usize>()
                .map_err(|_| "invalid datatype arity".to_string())?;
            Ok((name, arity))
        }
        _ => Err("invalid sort declaration".into()),
    }
}


/// 解析 (declare-const <name> <sort>)
/// 例如: (declare-const x List)
/// items[0] = "declare-const", items[1] = "x", items[2] = "List"
fn parse_declare_const(items: &[SExp]) -> Result<Command, String> {
    if items.len() != 3 {
        return Err("declare-const expects 2 arguments: name and sort".into());
    }
    let name = atom(&items[1])?.to_string();
    let sort = parse_sort(&items[2])?;
    Ok(Command::DeclareConst(name, sort))
}

/// 解析 (declare-fun <name> (<sort>*) <sort>)
/// 例如: (declare-fun f (Int List) Bool)
/// items[0] = "declare-fun", items[1] = "f", items[2] = (Int List), items[3] = "Bool"
///
/// 当参数列表为空时: (declare-fun y () Int)
/// 语义上等价于 (declare-const y Int)，但我们仍然保留为 DeclareFun 以区分来源
fn parse_declare_fun(items: &[SExp]) -> Result<Command, String> {
    if items.len() != 4 {
        return Err("declare-fun expects 3 arguments: name, arg sorts, and return sort".into());
    }
    let name = atom(&items[1])?.to_string();

    // 解析参数 sort 列表: items[2] 应该是一个 List，里面每个元素是 sort 表达式
    let arg_sorts = match &items[2] {
        SExp::List(xs) => {
            let mut sorts = Vec::new();
            for x in xs {
                sorts.push(parse_sort(x)?);
            }
            sorts
        }
        _ => return Err("declare-fun: expected a list of argument sorts".into()),
    };

    let ret_sort = parse_sort(&items[3])?;
    Ok(Command::DeclareFun(name, arg_sorts, ret_sort))
}


fn parse_constructors(sexp: &SExp) -> Result<Vec<ConstructorDecl>, String> {
    let xs = match sexp {
        SExp::List(xs) => xs,
        _ => return Err("expected constructor list".into()),
    };

    let mut ctors = Vec::new();
    for x in xs {
        ctors.push(parse_constructor(x)?);
    }
    Ok(ctors)
}

fn parse_constructor(sexp: &SExp) -> Result<ConstructorDecl, String> {
    match sexp {
        SExp::List(xs) if !xs.is_empty() => {
            let name = atom(&xs[0])?.to_string();
            let mut fields = Vec::new();
            for field in &xs[1..] {
                fields.push(parse_field(field)?);
            }
            Ok(ConstructorDecl { name, fields })
        }
        _ => Err("invalid constructor declaration".into()),
    }
}

/// 解析 sort 表达式: 原子 sort (如 Int, Bool, T) 或参数化 sort (如 (List Int))
fn parse_sort(sexp: &SExp) -> Result<SortExpr, String> {
    match sexp {
        SExp::Atom(s) => Ok(SortExpr::Simple(s.clone())),
        SExp::List(xs) if !xs.is_empty() => {
            let name = atom(&xs[0])?.to_string();
            let args: Result<Vec<SortExpr>, String> = xs[1..].iter().map(|x| parse_sort(x)).collect();
            Ok(SortExpr::Parametric(name, args?))
        }
        _ => Err(format!("invalid sort expression: {:?}", sexp)),
    }
}

fn parse_field(sexp: &SExp) -> Result<FieldDecl, String> {
    match sexp {
        SExp::List(xs) if xs.len() == 2 => Ok(FieldDecl {
            selector: atom(&xs[0])?.to_string(),
            sort: parse_sort(&xs[1])?,
        }),
        _ => Err("invalid field declaration".into()),
    }
}

fn atom(sexp: &SExp) -> Result<&str, String> {
    match sexp {
        SExp::Atom(s) => Ok(s),
        _ => Err(format!("expected atom, got {:?}", sexp)),
    }
}


/// 解析公式（Formula）——支持所有 QF_ADT 布尔连接词
///
/// SMT-LIB 中，公式（formula）就是 sort 为 Bool 的 term。
/// 这里我们用一个递归解析器来处理：
///   - 布尔常量: true, false
///   - 等式: (= t1 t2)
///   - 不等式: (distinct t1 t2 ...) => 脱糖为 And(Not(Eq)...)
///   - 逻辑连接词: (not φ), (and φ1 φ2 ...), (or φ1 φ2 ...), (=> φ1 φ2), (ite φ φ φ)
///   - ADT tester: (is-Cons x) => IsTester("Cons", x)
///   - 其他所有情况编码为 Eq(term, Var("true"))（Bool-sorted 变量或函数应用）
fn parse_formula(sexp: &SExp) -> Result<Formula, String> {
    match sexp {
        // ---- 原子（Atom）----
        SExp::Atom(s) => match s.as_str() {
            "true"  => Ok(Formula::True),
            "false" => Ok(Formula::False),
            // 其他原子视为 Bool-sorted 变量，编码为 Eq(term, Var("true"))
            // 例如 (declare-const p Bool) 后，(assert p) 中的 p
            _ => Ok(Formula::Eq(parse_term(sexp)?, Term::Var("true".to_string()))),
        },

        // ---- 列表（复合表达式）----
        SExp::List(xs) if !xs.is_empty() => {
            if let SExp::Atom(op) = &xs[0] {
                match op.as_str() {
                    // (= t1 t2)
                    "=" => {
                        if xs.len() != 3 {
                            return Err("= expects exactly 2 arguments".into());
                        }
                        Ok(Formula::Eq(parse_term(&xs[1])?, parse_term(&xs[2])?))
                    }

                    // (distinct t1 t2 ...) => 在 parse 时脱糖为 And([Not(Eq(ti,tj)), ...])
                    "distinct" => {
                        if xs.len() < 3 {
                            return Err("distinct expects at least 2 arguments".into());
                        }
                        let terms: Result<Vec<Term>, String> =
                            xs[1..].iter().map(|x| parse_term(x)).collect();
                        let terms = terms?;
                        let mut neqs = Vec::new();
                        for i in 0..terms.len() {
                            for j in (i + 1)..terms.len() {
                                neqs.push(Formula::Not(Box::new(
                                    Formula::Eq(terms[i].clone(), terms[j].clone()),
                                )));
                            }
                        }
                        if neqs.len() == 1 {
                            Ok(neqs.into_iter().next().unwrap())
                        } else {
                            Ok(Formula::And(neqs))
                        }
                    }

                    // (not φ)
                    "not" => {
                        if xs.len() != 2 {
                            return Err("not expects exactly 1 argument".into());
                        }
                        Ok(Formula::Not(Box::new(parse_formula(&xs[1])?)))
                    }

                    // (and φ1 φ2 ...)
                    "and" => {
                        if xs.len() < 3 {
                            return Err("and expects at least 2 arguments".into());
                        }
                        let formulas: Result<Vec<Formula>, String> =
                            xs[1..].iter().map(|x| parse_formula(x)).collect();
                        Ok(Formula::And(formulas?))
                    }

                    // (or φ1 φ2 ...)
                    "or" => {
                        if xs.len() < 3 {
                            return Err("or expects at least 2 arguments".into());
                        }
                        let formulas: Result<Vec<Formula>, String> =
                            xs[1..].iter().map(|x| parse_formula(x)).collect();
                        Ok(Formula::Or(formulas?))
                    }

                    // (=> φ1 φ2)
                    "=>" => {
                        if xs.len() != 3 {
                            return Err("=> expects exactly 2 arguments".into());
                        }
                        Ok(Formula::Implies(
                            Box::new(parse_formula(&xs[1])?),
                            Box::new(parse_formula(&xs[2])?),
                        ))
                    }

                    // (ite φ_cond φ_then φ_else)
                    // 公式级 ite: 条件和两个分支都递归解析为公式
                    // 注意：当分支不是布尔表达式时，会通过 Eq(term, Var("true")) 兜底
                    "ite" => {
                        if xs.len() != 4 {
                            return Err("ite expects exactly 3 arguments".into());
                        }
                        Ok(Formula::Ite(
                            Box::new(parse_formula(&xs[1])?),
                            Box::new(parse_formula(&xs[2])?),
                            Box::new(parse_formula(&xs[3])?),
                        ))
                    }

                    // ---- tester: (is-<Ctor> term) ----
                    // 当操作符以 "is-" 开头且恰好有 1 个参数时，
                    // 识别为 ADT tester，剥离 "is-" 前缀得到构造器名。
                    // 例如 (is-Cons x) => IsTester("Cons", parse_term(x))
                    _ if op.starts_with("is-") && xs.len() == 2 => {
                        let ctor_name = op[3..].to_string();
                        let term = parse_term(&xs[1])?;
                        Ok(Formula::IsTester(ctor_name, term))
                    }

                    // 其他所有函数应用: 编码为 Eq(term, Var("true"))
                    // 例如 (f 1 2) 其中 f: (Int Int) -> Bool
                    _ => Ok(Formula::Eq(parse_term(sexp)?, Term::Var("true".to_string()))),
                }
            } else {
                Err(format!("invalid formula: expected operator, got {:?}", sexp))
            }
        }

        _ => Err(format!("invalid formula: {:?}", sexp)),
    }
}

fn parse_term(sexp: &SExp) -> Result<Term, String> {
    match sexp {
        SExp::Atom(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Ok(Term::Int(n))
            } else {
                Ok(Term::Var(s.clone()))
            }
        }
        SExp::List(xs) if !xs.is_empty() => {
            // ---- 检查 head 是否是 (as func sort) — qualified function application ----
            // ((as func sort) arg1 arg2 ...) => As(App(func, args), sort)
            // ((as nil (List Int)))           => As(Var("nil"), (List Int))
            if let SExp::List(inner) = &xs[0] {
                if inner.len() == 3 {
                    if let Ok("as") = atom(&inner[0]) {
                        let func_name = atom(&inner[1])?.to_string();
                        let sort = parse_sort(&inner[2])?;
                        let mut args = Vec::new();
                        for x in &xs[1..] {
                            args.push(parse_term(x)?);
                        }
                        if args.is_empty() {
                            return Ok(Term::As(Box::new(Term::Var(func_name)), sort));
                        } else {
                            return Ok(Term::As(Box::new(Term::App(func_name, args)), sort));
                        }
                    }
                }
                return Err(format!("invalid term: {:?}", sexp));
            }

            let f = atom(&xs[0])?.to_string();

            // ---- as 特殊处理 ----
            // (as term sort) — 类型消歧（无参构造器常用）
            // 例如 (as nil (List Int)) => As(Var("nil"), Parametric("List",[Simple("Int")]))
            if f == "as" {
                if xs.len() != 3 {
                    return Err("as expects exactly 2 arguments: term and sort".into());
                }
                let inner = parse_term(&xs[1])?;
                let sort = parse_sort(&xs[2])?;
                return Ok(Term::As(Box::new(inner), sort));
            }

            // ---- ite 特殊处理 ----
            // (ite <condition> <then> <else>)
            // 条件用 parse_formula（保留 IsTester/And/Or 等结构），
            // 分支用 parse_term（它们是任意 sort 的 term）
            if f == "ite" {
                if xs.len() != 4 {
                    return Err("ite expects exactly 3 arguments".into());
                }
                return Ok(Term::Ite(
                    Box::new(parse_formula(&xs[1])?),
                    Box::new(parse_term(&xs[2])?),
                    Box::new(parse_term(&xs[3])?),
                ));
            }

            // ---- match 特殊处理 ----
            // (match <scrutinee> (pattern1 body1) (pattern2 body2) ...)
            // 在 parse 阶段仅记录 AST 结构，在 egg 编码时脱糖为 ite + tester + selector
            if f == "match" {
                if xs.len() < 3 {
                    return Err("match expects a scrutinee and at least one case".into());
                }
                let scrutinee = parse_term(&xs[1])?;
                let mut cases = Vec::new();
                for case_sexp in &xs[2..] {
                    cases.push(parse_match_case(case_sexp)?);
                }
                return Ok(Term::Match(Box::new(scrutinee), cases));
            }

            let mut args = Vec::new();
            for x in &xs[1..] {
                args.push(parse_term(x)?);
            }
            Ok(Term::App(f, args))
        }
        _ => Err(format!("invalid term: {:?}", sexp)),
    }
}

/// 解析 match 的一个分支: (pattern body)
/// 例如: (Nil 0) 或 ((Cons h t) (f h t))
fn parse_match_case(sexp: &SExp) -> Result<MatchCase, String> {
    match sexp {
        SExp::List(xs) if xs.len() == 2 => {
            let pattern = parse_match_pattern(&xs[0])?;
            let body = parse_term(&xs[1])?;
            Ok(MatchCase { pattern, body })
        }
        _ => Err(format!("invalid match case (expected 2-element list): {:?}", sexp)),
    }
}

/// 解析 match 模式
/// - 原子: Nil (无参构造器或 catch-all 变量)
/// - 列表: (Cons h t) (有参构造器 + 绑定变量)
fn parse_match_pattern(sexp: &SExp) -> Result<MatchPattern, String> {
    match sexp {
        SExp::Atom(name) => Ok(MatchPattern::Atom(name.clone())),
        SExp::List(xs) if !xs.is_empty() => {
            let ctor_name = atom(&xs[0])?.to_string();
            let vars: Result<Vec<String>, String> = xs[1..]
                .iter()
                .map(|x| atom(x).map(|s| s.to_string()))
                .collect();
            Ok(MatchPattern::Constructor(ctor_name, vars?))
        }
        _ => Err(format!("invalid match pattern: {:?}", sexp)),
    }
}


// ============================================================================
// AST -> egg e-graph 的转换（含布尔连接词支持）
// ============================================================================
//
// 整体思路：
//   1. 使用 egg::SymbolLang 作为 e-graph 的 Language。
//      SymbolLang 是 egg 提供的通用语言类型，每个节点由一个符号名(op)和子节点列表组成。
//
//   2. Term -> e-graph（与之前相同）：
//      - Term::Var("x")          -> 叶节点 "x"
//      - Term::Int(42)           -> 叶节点 "42"
//      - Term::App("f", [a, b])  -> 节点 "f"，子节点 [a, b]
//
//   3. Formula -> e-graph（新增布尔连接词编码）：
//      公式也被编码为 e-graph 节点，与 term 共享同一个 e-graph：
//      - Formula::True           -> 叶节点 "true"
//      - Formula::False          -> 叶节点 "false"
//      - Formula::Eq(t1, t2)    -> 节点 "="，子节点 [t1, t2]
//      - Formula::Not(φ)        -> 节点 "not"，子节点 [φ]
//      - Formula::And(φ1, φ2)   -> 节点 "and"，子节点 [φ1, φ2]（n-ary 折叠为二元）
//      - Formula::Or(φ1, φ2)    -> 节点 "or"，子节点 [φ1, φ2]
//      - Formula::Implies(φ,ψ)  -> 节点 "=>"，子节点 [φ, ψ]
//      - Formula::Ite(c,t,e)    -> 节点 "ite"，子节点 [c, t, e]
//      - Formula::IsTester(c,t) -> 节点 "is-C"，子节点 [t]
//
//   4. assert φ 的处理：
//      将 φ 编码到 e-graph 并 union(φ_id, true_id)，表示 φ 为真。
//      对顶层等式做额外优化：直接 union(t1, t2)。
//      对顶层 not-equal / distinct 记录 disequality。
//      对顶层 and 拆分为多个独立断言。
//
//   5. 布尔 rewrite rules：
//      自动生成标准的布尔化简规则（not/and/or/=>/ite 与 true/false 的交互），
//      让 egg 的 equality saturation 能自动推导布尔层面的等式。
//
//   6. check-sat：运行 saturation 后检查：
//      - true 和 false 是否被合并（全局矛盾）
//      - 所有 disequality 约束是否被违反

use egg::{Analysis, AstSize, DidMerge, EGraph, Extractor, Id, Language, Pattern, Rewrite, Runner, Symbol, SymbolLang};

// ============================================================================
// ADT Analysis: 构造子互斥性 (Disjointness) 与单射性 (Injectivity)
// ============================================================================
//
// 标准 ADT 理论要求:
//   1. 构造子互斥: 不同构造子产生不同值  C_i(...) ≠ C_j(...)  (i ≠ j)
//   2. 构造子单射: 相同构造子参数必须相同  C(a,b) = C(c,d) → a=c ∧ b=d
//
// 实现策略:
//   - make: 识别构造子节点，记录 (构造子名, 子节点列表)
//   - merge: 合并两个 e-class 时检测冲突（设 contradiction 标志）/ 记录单射推导
//   - 不使用 modify（避免在 egg rebuild 内部调用 union 导致死循环）
//   - 由 EggContext::rebuild_with_adt() 在 rebuild 外部循环应用 pending unions

/// 每个 e-class 的 ADT 分析数据
#[derive(Debug, Clone)]
struct AdtData {
    /// 如果该 e-class 包含某个构造子应用，记录 (构造子名, 子节点 Id 列表)
    /// 例如 Cons(id_1, id_2) → Some(("Cons", [id_1, id_2]))
    ///      Nil              → Some(("Nil", []))
    ///      变量 x / 整数 42  → None（不是构造子）
    constructor: Option<(Symbol, Vec<Id>)>,
}

/// ADT 自定义分析：在 e-class 合并时主动检测构造子冲突和单射性
#[derive(Debug, Clone)]
struct AdtAnalysis {
    /// 已知构造子名 → 参数个数 (arity)
    /// 在 generate_datatype_rules 中填充
    /// 例如: "Nil" → 0, "Cons" → 2, "Red" → 0
    ctor_arities: HashMap<String, usize>,

    /// 构造子互斥冲突标志
    /// 当 merge 发现同一 e-class 中有两个不同构造子时设为 true
    /// check_sat 检查此标志即可判定 unsat，无需令 true=false
    contradiction: bool,

    /// 待处理的 union 操作（仅单射性推导）
    /// 在 merge 中收集，由 EggContext::rebuild_with_adt() 在 rebuild 循环外应用
    pending_unions: Vec<(Id, Id)>,
}

impl Default for AdtAnalysis {
    fn default() -> Self {
        Self {
            ctor_arities: HashMap::new(),
            contradiction: false,
            pending_unions: Vec::new(),
        }
    }
}

impl Analysis<SymbolLang> for AdtAnalysis {
    type Data = AdtData;

    /// 为新加入 e-graph 的 e-node 计算 ADT 分析数据
    fn make(egraph: &EGraph<SymbolLang, Self>, enode: &SymbolLang) -> Self::Data {
        let name = enode.op.to_string();
        let children: Vec<Id> = enode.children().iter()
            .map(|&id| egraph.find(id))
            .collect();

        if let Some(&arity) = egraph.analysis.ctor_arities.get(&name) {
            if children.len() == arity {
                return AdtData {
                    constructor: Some((enode.op, children)),
                };
            }
        }

        AdtData { constructor: None }
    }

    /// 合并两个 e-class 的 ADT 数据
    ///
    /// 三种情况:
    ///   1. 双方都有构造子，且名字不同 → 设 contradiction = true
    ///   2. 双方都有构造子，且名字相同 → 单射性，推入子节点的 union 对
    ///   3. 仅一方有构造子 → 继承有构造子的那一方的信息
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        match (&a.constructor, &b.constructor) {
            (Some((name_a, children_a)), Some((name_b, children_b))) => {
                if name_a != name_b {
                    // ---- 构造子互斥 (Disjointness) ----
                    // 不同构造子被合并到同一 e-class → 矛盾
                    self.contradiction = true;
                    DidMerge(false, false)
                } else {
                    // ---- 构造子单射 (Injectivity) ----
                    // 相同构造子 C(a1,...,an) = C(b1,...,bn) → ai = bi
                    for (ca, cb) in children_a.iter().zip(children_b.iter()) {
                        if ca != cb {
                            self.pending_unions.push((*ca, *cb));
                        }
                    }
                    DidMerge(false, false)
                }
            }
            (None, Some(_)) => {
                // a 无构造子信息，b 有 → 继承 b 的信息
                a.constructor = b.constructor;
                DidMerge(true, false)
            }
            _ => {
                // 两者都没有，或 a 已有、b 没有 → 保持不变
                DidMerge(false, false)
            }
        }
    }

    // 不实现 modify — pending unions 由 EggContext::rebuild_with_adt() 处理
    // 在 modify 中调用 egraph.union() 可导致 egg rebuild 内部死循环
}

/// EggContext: 封装 e-graph 及相关状态
/// 这个结构体是 AST 世界和 egg 世界之间的桥梁。
/// 它持有 e-graph 本身、所有从 declare-datatypes 生成的 rewrite rules，
/// 以及从 assert(not (= ...)) 收集的不等式约束。
#[derive(Clone)]
struct EggContext {
    // egg 的核心数据结构：e-graph
    // SymbolLang 表示每个节点是一个 (符号名, 子节点列表) 的组合
    // () 是 Analysis 类型，这里我们不需要额外的分析数据，所以用单元类型
    egraph: EGraph<SymbolLang, AdtAnalysis>,

    // 从 declare-datatypes 自动生成的 rewrite rules
    // 例如：head(Cons(?x, ?y)) => ?x
    // 这些规则会在 equality saturation 阶段被反复应用
    rewrites: Vec<Rewrite<SymbolLang, AdtAnalysis>>,

    // 不等式约束列表
    // egg 原生只支持等式推理（union），不支持不等式
    // 所以我们把 NotEq 的两端 Id 记录下来，saturation 结束后手动检查
    disequalities: Vec<(Id, Id)>,

    // ---- 新增：用于 declare-const / declare-fun 的符号表 ----

    // 已声明的常量: name -> (sort, 在 e-graph 中的 Id)
    // declare-const 声明时就会在 e-graph 中创建叶节点，后续 assert 引用同名变量时
    // 会通过 egg 的去重机制自动指向同一个 e-class
    declared_consts: HashMap<String, (String, Id)>,

    // 已声明的函数: name -> (参数 sort 列表, 返回 sort)
    // declare-fun 只记录签名信息（arity 和 sort），不在 e-graph 中创建节点，
    // 因为函数只有在 assert 中被应用(App)时才会实际出现在 e-graph 中
    declared_funs: HashMap<String, (Vec<String>, String)>,

    // 所有已知的 sort 名称（来自 declare-datatypes 以及内建 sort 如 Int, Bool）
    known_sorts: Vec<String>,

    // ---- Tester 支持：构造器 → 数据类型 的反向映射 ----
    // 由 generate_datatype_rules 在处理 declare-datatypes 时填充。
    // 例如: "Nil" -> "List", "Cons" -> "List"
    // 用途：验证 tester 引用的构造器是否合法，以及查找同一数据类型下的兄弟构造器。
    ctor_to_dt: HashMap<String, String>,

    // ---- Match 支持：构造器 → selector 名称列表 ----
    // 由 generate_datatype_rules 在处理 declare-datatypes 时填充。
    // 例如: "Cons" -> ["head", "tail"]，"Nil" -> []
    // 用途：在 match 脱糖时，将 pattern 中的绑定变量替换为 selector 应用。
    ctor_selectors: HashMap<String, Vec<String>>,

    // ---- 新增：布尔常量在 e-graph 中的 Id ----
    // 在 EggContext 初始化时就创建 "true" 和 "false" 叶节点
    // assert φ 时，将 φ 的 Id 与 true_id 合并，表示 φ 为真
    // check-sat 时，检查 true_id 和 false_id 是否被合并到同一 e-class（矛盾 = unsat）
    true_id: Id,
    false_id: Id,

    // ---- Sort 类型检查环境 ----
    // 收集声明中的类型信息，在 assert 前验证公式类型正确性
    sort_env: SortEnv,

    // ---- check-sat 结果缓存 ----
    // 记录最近一次 check-sat 的结果，用于 get-model / get-value 的前置检查
    last_check_sat_result: Option<String>,
}

// ============================================================================
// Match 脱糖辅助函数：变量替换
// ============================================================================
//
// match 表达式中的 pattern 绑定变量（如 (Cons h t) 中的 h, t）需要被替换为
// 对应的 selector 应用（如 head(scrutinee), tail(scrutinee)）。
// 这组函数在 Term 和 Formula 上执行递归替换。

/// 在 Term 中将变量 `var` 替换为 `replacement`
fn subst_term(term: &Term, var: &str, replacement: &Term) -> Term {
    match term {
        Term::Var(name) if name == var => replacement.clone(),
        Term::Var(_) | Term::Int(_) => term.clone(),
        Term::App(f, args) => Term::App(
            f.clone(),
            args.iter().map(|a| subst_term(a, var, replacement)).collect(),
        ),
        Term::Ite(cond, then_t, else_t) => Term::Ite(
            Box::new(subst_formula(cond, var, replacement)),
            Box::new(subst_term(then_t, var, replacement)),
            Box::new(subst_term(else_t, var, replacement)),
        ),
        Term::Match(scrut, cases) => Term::Match(
            Box::new(subst_term(scrut, var, replacement)),
            cases.iter().map(|c| {
                // 如果 pattern 绑定了同名变量，则不替换该 case 的 body（变量被遮蔽）
                if match_pattern_binds(&c.pattern, var) {
                    c.clone()
                } else {
                    MatchCase {
                        pattern: c.pattern.clone(),
                        body: subst_term(&c.body, var, replacement),
                    }
                }
            }).collect(),
        ),
        Term::As(inner, sort_expr) => Term::As(
            Box::new(subst_term(inner, var, replacement)),
            sort_expr.clone(),
        ),
    }
}

/// 在 Formula 中将变量 `var` 替换为 `replacement`
fn subst_formula(formula: &Formula, var: &str, replacement: &Term) -> Formula {
    match formula {
        Formula::True | Formula::False => formula.clone(),
        Formula::Eq(t1, t2) => Formula::Eq(
            subst_term(t1, var, replacement),
            subst_term(t2, var, replacement),
        ),
        Formula::Not(f) => Formula::Not(Box::new(subst_formula(f, var, replacement))),
        Formula::And(fs) => Formula::And(
            fs.iter().map(|f| subst_formula(f, var, replacement)).collect(),
        ),
        Formula::Or(fs) => Formula::Or(
            fs.iter().map(|f| subst_formula(f, var, replacement)).collect(),
        ),
        Formula::Implies(l, r) => Formula::Implies(
            Box::new(subst_formula(l, var, replacement)),
            Box::new(subst_formula(r, var, replacement)),
        ),
        Formula::Ite(c, t, e) => Formula::Ite(
            Box::new(subst_formula(c, var, replacement)),
            Box::new(subst_formula(t, var, replacement)),
            Box::new(subst_formula(e, var, replacement)),
        ),
        Formula::IsTester(ctor, t) => Formula::IsTester(
            ctor.clone(),
            subst_term(t, var, replacement),
        ),
    }
}

/// 检查 match pattern 是否绑定了指定变量名（用于遮蔽检测）
fn match_pattern_binds(pattern: &MatchPattern, var: &str) -> bool {
    match pattern {
        MatchPattern::Constructor(_, vars) => vars.iter().any(|v| v == var),
        MatchPattern::Atom(name) => name == var,
    }
}

impl EggContext {
    /// 创建一个新的空 EggContext
    /// 初始化时：
    ///   1. 在 e-graph 中创建 "true" 和 "false" 叶节点
    ///   2. 自动生成布尔化简的 rewrite rules
    ///   3. 预注册内建 sort: Int 和 Bool
    fn new() -> Self {
        let mut egraph = EGraph::new(AdtAnalysis::default());
        let true_id = egraph.add(SymbolLang::leaf("true"));
        let false_id = egraph.add(SymbolLang::leaf("false"));

        let mut ctx = EggContext {
            egraph,
            rewrites: Vec::new(),
            disequalities: Vec::new(),
            declared_consts: HashMap::new(),
            declared_funs: HashMap::new(),
            known_sorts: vec!["Int".to_string(), "Bool".to_string()],
            ctor_to_dt: HashMap::new(),
            ctor_selectors: HashMap::new(),
            true_id,
            false_id,
            sort_env: SortEnv::new(),
            last_check_sat_result: None,
        };
        ctx.generate_boolean_rules();
        ctx
    }

    /// 带 ADT 单射性传播的 rebuild
    ///
    /// egg 的 rebuild() 只处理 congruence closure 和 analysis 更新，
    /// 但不会自动应用 analysis.pending_unions（因为我们不使用 modify）。
    /// 此方法在 rebuild 后检查并应用 pending unions，循环直到收敛。
    ///
    /// 流程:
    ///   1. egraph.rebuild() — 处理 union-find pending + congruence closure
    ///   2. 检查 analysis.pending_unions（由 merge 中的单射性检测填充）
    ///   3. 若非空: 应用所有 pending unions 并回到步骤 1
    ///   4. 若为空: 收敛，返回
    fn rebuild_with_adt(&mut self) {
        loop {
            self.egraph.rebuild();
            let pending: Vec<(Id, Id)> =
                self.egraph.analysis.pending_unions.drain(..).collect();
            if pending.is_empty() {
                break;
            }
            for (a, b) in pending {
                self.egraph.union(a, b);
            }
            // 新的 union 可能触发更多 merge → 更多 pending，循环处理
        }
    }

    /// 生成标准的布尔化简 rewrite rules
    ///
    /// 这些规则让 egg 在 equality saturation 阶段能自动推导布尔层面的等式。
    /// 每条规则的含义：
    ///   - not(true) => false, not(false) => true     （否定常量）
    ///   - not(not(?x)) => ?x                         （双重否定消除）
    ///   - and(true,?x) => ?x, and(false,?x) => false （与的单位元和零元）
    ///   - or(false,?x) => ?x, or(true,?x) => true    （或的单位元和零元）
    ///   - =>(true,?x) => ?x, =>(false,?x) => true    （蕴含化简）
    ///   - ite(true,?x,?y) => ?x, ite(false,?x,?y) => ?y  （条件化简）
    fn generate_boolean_rules(&mut self) {
        let rules: Vec<(&str, &str, &str)> = vec![
            // ---- not ----
            ("not-true",    "(not true)",           "false"),
            ("not-false",   "(not false)",          "true"),
            ("not-not",     "(not (not ?x))",       "?x"),
            // ---- and ----
            ("and-true-l",  "(and true ?x)",        "?x"),
            ("and-true-r",  "(and ?x true)",        "?x"),
            ("and-false-l", "(and false ?x)",       "false"),
            ("and-false-r", "(and ?x false)",       "false"),
            // ---- or ----
            ("or-false-l",  "(or false ?x)",        "?x"),
            ("or-false-r",  "(or ?x false)",        "?x"),
            ("or-true-l",   "(or true ?x)",         "true"),
            ("or-true-r",   "(or ?x true)",         "true"),
            // ---- implies (=>) ----
            ("imp-true",    "(=> true ?x)",         "?x"),
            ("imp-false",   "(=> false ?x)",        "true"),
            ("imp-to-true", "(=> ?x true)",         "true"),
            // ---- ite ----
            ("ite-true",    "(ite true ?x ?y)",     "?x"),
            ("ite-false",   "(ite false ?x ?y)",    "?y"),
            // ---- 等式自反性 ----
            // 当 (= t t) 的两个子节点在同一 e-class 时，该等式为 true
            ("eq-refl",     "(= ?x ?x)",            "true"),
        ];

        println!("[egg] 生成布尔化简 rewrite rules ({} 条):", rules.len());
        for (name, lhs_str, rhs_str) in &rules {
            let lhs: Pattern<SymbolLang> = lhs_str.parse()
                .unwrap_or_else(|e| panic!("无法解析布尔规则 lhs '{}': {}", lhs_str, e));
            let rhs: Pattern<SymbolLang> = rhs_str.parse()
                .unwrap_or_else(|e| panic!("无法解析布尔规则 rhs '{}': {}", rhs_str, e));
            let rule: Rewrite<SymbolLang, AdtAnalysis> = Rewrite::new(*name, lhs, rhs)
                .unwrap_or_else(|e| panic!("无法创建布尔规则 '{}': {}", name, e));
            println!("  {} => {}", lhs_str, rhs_str);
            self.rewrites.push(rule);
        }
    }

    /// 将 match 表达式脱糖为嵌套的 Term::Ite
    ///
    /// 脱糖过程：
    ///   (match x (Nil 0) ((Cons h t) (App f h)))
    ///   => (ite (is-Nil x) 0 (ite (is-Cons x) (f (head x)) ???))
    ///
    /// 对于最后一个 case，直接使用其 body 作为 else 分支（无需 ite 包装）。
    /// 对于带绑定变量的 pattern（如 (Cons h t)），将 h, t 替换为 selector 应用
    /// （如 head(x), tail(x)），替换使用 subst_term。
    fn desugar_match(&self, scrutinee: &Term, cases: &[MatchCase]) -> Term {
        assert!(!cases.is_empty(), "match 至少需要一个 case");

        let case = &cases[0];
        let body = self.substitute_pattern(scrutinee, &case.pattern, &case.body);

        if cases.len() == 1 {
            // 最后一个 case：直接返回替换后的 body
            body
        } else {
            let else_body = self.desugar_match(scrutinee, &cases[1..]);
            match &case.pattern {
                MatchPattern::Constructor(ctor_name, _) => Term::Ite(
                    Box::new(Formula::IsTester(ctor_name.clone(), scrutinee.clone())),
                    Box::new(body),
                    Box::new(else_body),
                ),
                MatchPattern::Atom(name) => {
                    if self.ctor_to_dt.contains_key(name) {
                        // 无参构造器（如 Nil）
                        Term::Ite(
                            Box::new(Formula::IsTester(name.clone(), scrutinee.clone())),
                            Box::new(body),
                            Box::new(else_body),
                        )
                    } else {
                        // 变量/通配符 — 匹配一切，作为 catch-all
                        body
                    }
                }
            }
        }
    }

    /// 对 match case 的 body 执行 pattern 绑定变量替换
    ///
    /// 将 pattern 中的绑定变量替换为对应的 selector 应用。
    /// 例如: pattern = (Cons h t), scrutinee = x
    ///   => h 替换为 App("head", [x])
    ///   => t 替换为 App("tail", [x])
    fn substitute_pattern(&self, scrutinee: &Term, pattern: &MatchPattern, body: &Term) -> Term {
        match pattern {
            MatchPattern::Constructor(ctor_name, vars) => {
                let selectors = self.ctor_selectors.get(ctor_name)
                    .unwrap_or_else(|| panic!("match 中遇到未知构造器: {}", ctor_name));
                assert_eq!(
                    vars.len(), selectors.len(),
                    "构造器 {} 的 pattern 参数数量 ({}) 与 field 数量 ({}) 不匹配",
                    ctor_name, vars.len(), selectors.len()
                );
                let mut result = body.clone();
                for (var, sel) in vars.iter().zip(selectors.iter()) {
                    let replacement = Term::App(sel.clone(), vec![scrutinee.clone()]);
                    result = subst_term(&result, var, &replacement);
                }
                result
            }
            MatchPattern::Atom(name) => {
                if self.ctor_to_dt.contains_key(name) {
                    // 无参构造器 — 无绑定变量
                    body.clone()
                } else {
                    // 变量绑定 — 将 name 替换为 scrutinee
                    subst_term(body, name, scrutinee)
                }
            }
        }
    }

    /// 将一个 AST Term 递归地添加到 e-graph 中，返回该 term 在 e-graph 中的 Id
    ///
    /// 这是 AST -> egg 转换的核心函数。
    /// 对应关系：
    ///   Term::Var("x")          => SymbolLang::leaf("x")           叶节点
    ///   Term::Int(42)           => SymbolLang::leaf("42")          叶节点（整数转为字符串）
    ///   Term::App("f", [a, b])  => SymbolLang::new("f", [id_a, id_b])  内部节点
    ///   Term::Match(s, cases)   => 脱糖为嵌套 ite 后递归编码
    ///
    /// 递归过程：对于 App，先递归添加所有子 term 拿到它们的 Id，
    /// 然后用这些 Id 构造当前节点并添加到 e-graph。
    fn add_term(&mut self, term: &Term) -> Id {
        match term {
            Term::Var(name) => {
                // 变量：作为叶节点添加
                // 例如 Nil 会变成一个 op="Nil"、无子节点的 e-node
                let node = SymbolLang::leaf(name.as_str());
                self.egraph.add(node)
            }
            Term::Int(n) => {
                // 整数字面量：转为字符串后作为叶节点
                // 例如 1 会变成 op="1"、无子节点的 e-node
                let node = SymbolLang::leaf(&n.to_string());
                self.egraph.add(node)
            }
            Term::App(func, args) => {
                // 函数应用：先递归处理所有参数，收集它们的 Id
                let child_ids: Vec<Id> = args.iter().map(|a| self.add_term(a)).collect();
                // 然后构造当前节点：op = func, children = child_ids
                // 例如 Cons(1, Nil) 会变成 op="Cons", children=[id_of_1, id_of_Nil]
                let node = SymbolLang::new(func.as_str(), child_ids);
                self.egraph.add(node)
            }
            Term::Ite(cond, then_t, else_t) => {
                // Term 级 ite: 条件走 add_formula，分支走 add_term
                // 在 e-graph 中编码为 SymbolLang::new("ite", [cond_id, then_id, else_id])
                // 与 Formula::Ite 共享同一个 "ite" 操作符名，
                // 因此同一套 rewrite rules（ite-true / ite-false）对两者都生效
                let cond_id = self.add_formula(cond);
                let then_id = self.add_term(then_t);
                let else_id = self.add_term(else_t);
                let node = SymbolLang::new("ite", vec![cond_id, then_id, else_id]);
                self.egraph.add(node)
            }
            Term::Match(scrutinee, cases) => {
                // Match 脱糖：将 match 表达式转为嵌套 ite + tester + selector，
                // 然后递归调用 add_term 编码到 e-graph。
                // 例如: (match x (Nil 0) ((Cons h t) h))
                //   => (ite (is-Nil x) 0 (head x))
                let desugared = self.desugar_match(scrutinee, cases);
                println!("[egg]   match 脱糖结果: {:?}", desugared);
                self.add_term(&desugared)
            }
            Term::As(inner, _sort_expr) => {
                // Sort 擦除: (as term sort) 在 egg 编码时仅编码内部 term
                // sort 信息仅用于类型检查阶段，rewrite rules 是 sort 无关的
                self.add_term(inner)
            }
        }
    }

    /// 处理 declare-const: 在 e-graph 中注册一个常量（叶节点）
    ///
    /// (declare-const x List) 的含义：
    ///   - x 是一个 sort 为 List 的未解释常量
    ///   - 在 e-graph 中，x 被表示为一个叶节点 SymbolLang::leaf("x")
    ///   - 后续 assert 中出现的 x 会通过 egg 的结构共享自动指向这个 e-class
    ///
    /// 为什么要在声明时就加入 e-graph？
    ///   因为 SMT-LIB 语义要求已声明的常量在整个上下文中有唯一的解释。
    ///   提前注册可以确保所有引用都指向同一个 e-class，
    ///   同时我们在符号表中记录 sort 信息以供后续类型检查使用。
    fn process_declare_const(&mut self, name: &str, sort: &str) {
        // 在 e-graph 中创建叶节点
        let node = SymbolLang::leaf(name);
        let id = self.egraph.add(node);
        // 记录到符号表
        self.declared_consts
            .insert(name.to_string(), (sort.to_string(), id));
        println!(
            "[egg] declare-const: 常量 '{}' (sort: {}) 注册为 e-graph 叶节点, id = {:?}",
            name, sort, id
        );
    }

    /// 处理 declare-fun: 记录函数签名到符号表
    ///
    /// (declare-fun f (Int List) Bool) 的含义：
    ///   - f 是一个接受 (Int, List) 返回 Bool 的未解释函数
    ///   - 函数本身不在 e-graph 中创建节点（只有被 apply 时才会出现）
    ///   - 但我们需要记录其 arity 和 sort 签名
    ///
    /// 特殊情况：当参数列表为空时 (declare-fun y () Int)
    ///   - 语义等价于 (declare-const y Int)
    ///   - 这种情况下我们同时在 e-graph 中创建叶节点
    fn process_declare_fun(&mut self, name: &str, arg_sorts: &[String], ret_sort: &str) {
        if arg_sorts.is_empty() {
            // 0 元函数 = 常量，直接在 e-graph 中创建叶节点
            let node = SymbolLang::leaf(name);
            let id = self.egraph.add(node);
            self.declared_consts
                .insert(name.to_string(), (ret_sort.to_string(), id));
            println!(
                "[egg] declare-fun: '{}' 参数列表为空，等价于 declare-const (sort: {}), id = {:?}",
                name, ret_sort, id
            );
        } else {
            // 有参数的函数：只记录签名，不创建 e-graph 节点
            self.declared_funs.insert(
                name.to_string(),
                (arg_sorts.to_vec(), ret_sort.to_string()),
            );
            println!(
                "[egg] declare-fun: 函数 '{}' ({}) -> {} 已注册 (arity = {})",
                name,
                arg_sorts.join(", "),
                ret_sort,
                arg_sorts.len()
            );
        }
    }

    /// 将一个 AST Formula 递归地编码到 e-graph 中，返回该公式在 e-graph 中的 Id
    ///
    /// 这是 Formula -> egg 转换的核心函数。
    /// 公式和 term 共享同一个 e-graph，布尔连接词被编码为普通的 SymbolLang 节点：
    ///   Formula::True             => 返回预先创建的 true_id
    ///   Formula::False            => 返回预先创建的 false_id
    ///   Formula::Eq(t1, t2)      => 节点 op="=", children=[add_term(t1), add_term(t2)]
    ///   Formula::Not(φ)          => 节点 op="not", children=[add_formula(φ)]
    ///   Formula::And([φ1,φ2,...]) => 折叠为二元: (and φ1 (and φ2 ...))
    ///   Formula::Or([φ1,φ2,...])  => 折叠为二元: (or φ1 (or φ2 ...))
    ///   Formula::Implies(φ,ψ)    => 节点 op="=>", children=[φ, ψ]
    ///   Formula::Ite(c,t,e)      => 节点 op="ite", children=[c, t, e]
    ///   Formula::IsTester(c, t)  => 节点 "is-C"，子节点 [add_term(t)]
    fn add_formula(&mut self, formula: &Formula) -> Id {
        match formula {
            Formula::True => self.true_id,
            Formula::False => self.false_id,

            Formula::Eq(t1, t2) => {
                let id1 = self.add_term(t1);
                let id2 = self.add_term(t2);
                let node = SymbolLang::new("=", vec![id1, id2]);
                self.egraph.add(node)
            }

            Formula::Not(inner) => {
                let inner_id = self.add_formula(inner);
                let node = SymbolLang::new("not", vec![inner_id]);
                self.egraph.add(node)
            }

            Formula::And(formulas) => {
                // n-ary and 折叠为右结合的二元 and: (and φ1 (and φ2 (and φ3 φ4)))
                // 这样布尔 rewrite rules（都是二元）可以直接匹配
                let ids: Vec<Id> = formulas.iter().map(|f| self.add_formula(f)).collect();
                let mut result = *ids.last().unwrap();
                for &id in ids[..ids.len() - 1].iter().rev() {
                    let node = SymbolLang::new("and", vec![id, result]);
                    result = self.egraph.add(node);
                }
                result
            }

            Formula::Or(formulas) => {
                // 同 And，折叠为右结合的二元 or
                let ids: Vec<Id> = formulas.iter().map(|f| self.add_formula(f)).collect();
                let mut result = *ids.last().unwrap();
                for &id in ids[..ids.len() - 1].iter().rev() {
                    let node = SymbolLang::new("or", vec![id, result]);
                    result = self.egraph.add(node);
                }
                result
            }

            Formula::Implies(lhs, rhs) => {
                let lhs_id = self.add_formula(lhs);
                let rhs_id = self.add_formula(rhs);
                let node = SymbolLang::new("=>", vec![lhs_id, rhs_id]);
                self.egraph.add(node)
            }

            Formula::Ite(cond, then_f, else_f) => {
                let cond_id = self.add_formula(cond);
                let then_id = self.add_formula(then_f);
                let else_id = self.add_formula(else_f);
                let node = SymbolLang::new("ite", vec![cond_id, then_id, else_id]);
                self.egraph.add(node)
            }

            Formula::IsTester(ctor_name, term) => {
                // IsTester: (is-Cons x) 编码为 e-graph 节点 op="is-Cons", children=[x]
                // egg 的 rewrite rules（在 generate_datatype_rules 中生成）会处理化简：
                //   (is-Cons (Cons ?x0 ?x1)) => true   （正向 tester）
                //   (is-Nil  (Cons ?x0 ?x1)) => false   （反向 tester）
                let term_id = self.add_term(term);
                let tester_op = format!("is-{}", ctor_name);
                let node = SymbolLang::new(tester_op, vec![term_id]);
                self.egraph.add(node)
            }
        }
    }

    /// 处理一条 assert 命令中的 Formula
    ///
    /// 核心语义: (assert φ) 意味着 φ 为真。
    ///
    /// 实现策略（两层处理）：
    ///   1. 通用处理：将 φ 编码到 e-graph，然后 union(φ_id, true_id)
    ///   2. 特殊优化：对特定顶层结构做直接处理以增强推理能力
    ///      - Eq: 直接 union 两个 term（比仅标记 "= node" 为 true 更强）
    ///      - Not(Eq): 记录 disequality
    ///      - And: 拆分为多个独立断言（因为 egg 无法从 "and(a,b)=true" 推出 "a=true"）
    fn process_assertion(&mut self, formula: &Formula) {
        // 第一层：通用编码 —— 将公式添加到 e-graph 并声明为 true
        let formula_id = self.add_formula(formula);
        self.egraph.union(formula_id, self.true_id);
        self.rebuild_with_adt();
        println!("[egg] 断言公式已编码到 e-graph, union({:?}, true)", formula_id);

        // 第二层：针对特定结构的直接优化处理
        self.process_assertion_direct(formula);
    }

    /// 对公式结构做直接处理（优化层）
    ///
    /// 这一层是必要的，因为 egg 的 rewrite rules 只能做模式匹配和替换，
    /// 无法表达 "如果 (and a b) = true，则 a = true 且 b = true" 这样的条件推理。
    /// 所以我们在 Rust 层面手动处理这些关键情况。
    fn process_assertion_direct(&mut self, formula: &Formula) {
        match formula {
            Formula::Eq(t1, t2) => {
                // 等式断言：直接 union 两个 term
                let id1 = self.add_term(t1);
                let id2 = self.add_term(t2);
                self.egraph.union(id1, id2);
                self.rebuild_with_adt();
                println!("[egg]   优化: 直接 union({:?}, {:?})", id1, id2);
            }

            Formula::Not(inner) => {
                if let Formula::Eq(t1, t2) = inner.as_ref() {
                    // (not (= t1 t2)) => 记录 disequality
                    let id1 = self.add_term(t1);
                    let id2 = self.add_term(t2);
                    self.disequalities.push((id1, id2));
                    println!("[egg]   优化: 记录不等式 {:?} != {:?}", id1, id2);
                }
            }

            Formula::And(conjuncts) => {
                // 顶层 and 拆分：递归地将每个子公式也断言为 true
                println!("[egg]   优化: 拆分 and ({} 个子公式)", conjuncts.len());
                for conj in conjuncts {
                    let conj_id = self.add_formula(conj);
                    self.egraph.union(conj_id, self.true_id);
                    self.rebuild_with_adt();
                    // 递归处理子公式中可能的嵌套 Eq/Not/And 等
                    self.process_assertion_direct(conj);
                }
            }

            // 其他公式结构依赖 rewrite rules 来化简，不做额外处理
            _ => {}
        }
    }

    /// 根据 declare-datatypes 中的构造器声明，生成 selector rewrite rules
    ///
    /// 原理：对于构造器 (Cons (head Int) (tail List))，它有两个 field：
    ///   - head 是第 0 个 field 的 selector
    ///   - tail 是第 1 个 field 的 selector
    ///
    /// 我们需要生成的规则是：
    ///   head(Cons(?x0, ?x1)) => ?x0    （head 提取 Cons 的第一个参数）
    ///   tail(Cons(?x0, ?x1)) => ?x1    （tail 提取 Cons 的第二个参数）
    ///
    /// 这些规则用 egg 的 Pattern 语法表示：
    ///   lhs: "(head (Cons ?x0 ?x1))"
    ///   rhs: "?x0"
    ///
    /// egg 的 equality saturation 引擎会自动匹配这些 pattern 并应用 rewrite。
    fn generate_datatype_rules(&mut self, datatypes: &[DatatypeDecl]) {
        for dt in datatypes {
            // 将这个数据类型的名字注册到已知 sort 列表中
            if !self.known_sorts.contains(&dt.name) {
                self.known_sorts.push(dt.name.clone());
            }

            // 注册所有构造器到 ctor_to_dt 反向映射，同时记录 selector 名称列表
            for ctor in &dt.constructors {
                self.ctor_to_dt.insert(ctor.name.clone(), dt.name.clone());
                let selectors: Vec<String> = ctor.fields.iter()
                    .map(|f| f.selector.clone())
                    .collect();
                self.ctor_selectors.insert(ctor.name.clone(), selectors);

                // 注册构造器到 AdtAnalysis 的 ctor_arities，
                // 使得 make() 能识别构造子节点，merge() 能检测互斥/单射
                self.egraph.analysis.ctor_arities
                    .insert(ctor.name.clone(), ctor.fields.len());
            }

            println!("[egg] 为数据类型 '{}' 生成 rewrite rules:", dt.name);

            // ============================================================
            // 第一部分：Selector 规则（原有逻辑）
            // ============================================================
            // 对每个有参构造器的每个 field，生成 selector(Ctor(?x0,...)) => ?xi

            for ctor in &dt.constructors {
                if ctor.fields.is_empty() {
                    println!("  构造器 '{}': 无参数，无需生成 selector 规则", ctor.name);
                    continue;
                }

                let var_names: Vec<String> = (0..ctor.fields.len())
                    .map(|i| format!("?x{}", i))
                    .collect();

                let ctor_pattern = format!(
                    "({} {})",
                    ctor.name,
                    var_names.join(" ")
                );

                for (i, field) in ctor.fields.iter().enumerate() {
                    let lhs_str = format!("({} {})", field.selector, ctor_pattern);
                    let rhs_str = var_names[i].clone();

                    println!("  生成 selector 规则: {} => {}", lhs_str, rhs_str);

                    let lhs: Pattern<SymbolLang> = lhs_str.parse()
                        .expect(&format!("无法解析 lhs pattern: {}", lhs_str));
                    let rhs: Pattern<SymbolLang> = rhs_str.parse()
                        .expect(&format!("无法解析 rhs pattern: {}", rhs_str));

                    let rule_name = format!("{}_{}", field.selector, ctor.name);
                    let rule: Rewrite<SymbolLang, AdtAnalysis> =
                        Rewrite::new(rule_name, lhs, rhs)
                            .expect("无法创建 rewrite rule");

                    self.rewrites.push(rule);
                }
            }

            // ============================================================
            // 第二部分：Tester 规则（新增）
            // ============================================================
            // 对每个构造器 Ci，生成：
            //   正向 tester: (is-Ci (Ci ?x0 ?x1 ...)) => true
            //   反向 tester: 对同一数据类型中每个其他构造器 Cj (j≠i):
            //                (is-Ci (Cj ?y0 ?y1 ...)) => false
            //
            // 例如 List 有 Nil (0参) 和 Cons (2参):
            //   (is-Nil Nil)              => true     正向
            //   (is-Cons (Cons ?x0 ?x1))  => true     正向
            //   (is-Nil (Cons ?x0 ?x1))   => false    反向 (Nil 的 tester 碰到 Cons)
            //   (is-Cons Nil)             => false    反向 (Cons 的 tester 碰到 Nil)

            // 先为每个构造器预计算其 pattern 字符串
            // 无参构造器: "Nil" (叶节点)
            // 有参构造器: "(Cons ?x0 ?x1)" (内部节点)
            // 注意: 不同构造器的变量名用不同前缀避免冲突
            let ctor_patterns: Vec<String> = dt.constructors.iter().map(|ctor| {
                if ctor.fields.is_empty() {
                    ctor.name.clone()
                } else {
                    let vars: Vec<String> = (0..ctor.fields.len())
                        .map(|i| format!("?v{}", i))
                        .collect();
                    format!("({} {})", ctor.name, vars.join(" "))
                }
            }).collect();

            for (i, ci) in dt.constructors.iter().enumerate() {
                let tester_name = format!("is-{}", ci.name);

                // 正向 tester: (is-Ci <Ci_pattern>) => true
                let pos_lhs = format!("({} {})", tester_name, ctor_patterns[i]);
                let pos_rhs = "true";
                println!("  生成 tester 规则 (正向): {} => {}", pos_lhs, pos_rhs);

                let lhs: Pattern<SymbolLang> = pos_lhs.parse()
                    .expect(&format!("无法解析 tester lhs: {}", pos_lhs));
                let rhs: Pattern<SymbolLang> = pos_rhs.parse().unwrap();
                let rule_name = format!("tester-pos-{}", ci.name);
                let rule: Rewrite<SymbolLang, AdtAnalysis> =
                    Rewrite::new(rule_name, lhs, rhs).expect("无法创建 tester rule");
                self.rewrites.push(rule);

                // 反向 tester: 对每个 Cj (j≠i), (is-Ci <Cj_pattern>) => false
                for (j, cj) in dt.constructors.iter().enumerate() {
                    if j == i {
                        continue;
                    }
                    let neg_lhs = format!("({} {})", tester_name, ctor_patterns[j]);
                    let neg_rhs = "false";
                    println!("  生成 tester 规则 (反向): {} => {}", neg_lhs, neg_rhs);

                    let lhs: Pattern<SymbolLang> = neg_lhs.parse()
                        .expect(&format!("无法解析 tester lhs: {}", neg_lhs));
                    let rhs: Pattern<SymbolLang> = neg_rhs.parse().unwrap();
                    let rule_name = format!("tester-neg-{}-{}", ci.name, cj.name);
                    let rule: Rewrite<SymbolLang, AdtAnalysis> =
                        Rewrite::new(rule_name, lhs, rhs).expect("无法创建 tester rule");
                    self.rewrites.push(rule);
                }
            }
        }
    }

    /// 运行 equality saturation 并检查可满足性
    ///
    /// 流程：
    ///   1. 用 egg::Runner 运行 equality saturation。
    ///      Runner 会反复应用所有 rewrite rules，直到 e-graph 不再变化（饱和）或达到上限。
    ///   2. saturation 完成后，检查所有 disequality 约束。
    ///   3. 如果某对 (id1, id2) 在 e-graph 中属于同一个 e-class（find(id1) == find(id2)），
    ///      说明 equality saturation 推导出它们相等，但我们断言它们不等，产生矛盾 -> unsat。
    ///   4. 如果所有不等式约束都没有矛盾 -> sat（注意：这只是一个简化判断，
    ///      完整的 SMT solver 需要更复杂的决策过程）。
    fn check_sat(&mut self) -> String {
        println!("\n[egg] 开始运行 equality saturation...");
        println!("[egg] 共有 {} 条 rewrite rules", self.rewrites.len());

        // 创建 Runner 并运行 equality saturation
        // Runner 会自动管理迭代过程：匹配 pattern -> 应用 rewrite -> rebuild -> 重复
        // iter_limit: 最大迭代次数，防止无限循环
        // node_limit: e-graph 最大节点数，防止爆炸式增长
        let runner = Runner::default()
            .with_egraph(std::mem::take(&mut self.egraph)) // 把 e-graph 移交给 runner
            .with_iter_limit(100)
            .with_node_limit(10_000)
            .run(&self.rewrites); // 运行所有 rewrite rules

        // saturation 完成后，把 e-graph 拿回来
        self.egraph = runner.egraph;

        // 处理 Runner 期间积累的 ADT 单射性 pending unions
        self.rebuild_with_adt();

        println!(
            "[egg] Equality saturation 完成。e-graph 中共有 {} 个 e-class",
            self.egraph.number_of_classes()
        );

        // ---- 检查 0: 构造子互斥冲突 ----
        // AdtAnalysis 的 merge 在发现同一 e-class 中有不同构造子时设 contradiction 标志
        if self.egraph.analysis.contradiction {
            println!("[egg] 发现矛盾！构造子互斥冲突（不同构造子被合并到同一 e-class）。");
            self.last_check_sat_result = Some("unsat".to_string());
            return "unsat".to_string();
        }

        // ---- 检查 1: true 和 false 是否被合并（全局矛盾）----
        // 如果 equality saturation 推导出 true = false，说明断言集合本身就矛盾
        let true_canon = self.egraph.find(self.true_id);
        let false_canon = self.egraph.find(self.false_id);
        println!(
            "[egg] 检查全局一致性: find(true) = {:?}, find(false) = {:?}",
            true_canon, false_canon
        );
        if true_canon == false_canon {
            println!("[egg] 发现矛盾！true 和 false 被合并到同一个 e-class。");
            self.last_check_sat_result = Some("unsat".to_string());
            return "unsat".to_string();
        }

        // ---- 检查 2: 所有不等式约束 ----
        for &(id1, id2) in &self.disequalities {
            // find(id) 返回该 Id 所在 e-class 的规范 Id（canonical Id）
            // 如果两个 Id 的 find 结果相同，说明它们在同一个 e-class 中，即被证明相等
            let canon1 = self.egraph.find(id1);
            let canon2 = self.egraph.find(id2);

            println!(
                "[egg] 检查不等式: find({:?}) = {:?}, find({:?}) = {:?}",
                id1, canon1, id2, canon2
            );

            if canon1 == canon2 {
                // 矛盾！我们断言 t1 != t2，但 e-graph 推导出 t1 == t2
                println!("[egg] 发现矛盾！不等式约束被违反。");
                self.last_check_sat_result = Some("unsat".to_string());
                return "unsat".to_string();
            }
        }

        // 所有检查通过
        self.last_check_sat_result = Some("sat".to_string());
        "sat".to_string()
    }

    /// 提取模型: 对每个已声明常量，从 e-graph 中提取最简 term 作为其值
    ///
    /// 使用 egg 的 Extractor + AstSize 代价函数，选择最小（最简洁）的表示。
    /// 输出格式遵循 SMT-LIB 标准: (model (define-fun name () sort value) ...)
    fn get_model(&self) -> Result<String, String> {
        match &self.last_check_sat_result {
            Some(r) if r == "sat" => {}
            Some(r) => return Err(format!("get-model: 最近 check-sat 返回 {}, 非 sat", r)),
            None => return Err("get-model: 尚未执行 check-sat".into()),
        }

        let extractor = Extractor::new(&self.egraph, AstSize);
        let mut model_lines = Vec::new();

        // 按名称排序以获得稳定输出
        let mut consts: Vec<_> = self.declared_consts.iter().collect();
        consts.sort_by_key(|(name, _)| (*name).clone());

        for (name, (sort, id)) in &consts {
            let (_cost, best_expr) = extractor.find_best(*id);
            model_lines.push(format!("  (define-fun {} () {} {})", name, sort, best_expr));
        }

        Ok(format!("(\n{}\n)", model_lines.join("\n")))
    }

    /// 查询指定 term 的值: 将 term 添加到 e-graph 并提取其最简等价形式
    ///
    /// 两趟处理:
    ///   Pass 1: 将所有查询 term 加入 e-graph（可能创建新节点），然后 rebuild
    ///   Pass 2: 用 Extractor 提取每个 term 的最简等价表示
    fn get_value(&mut self, terms: &[Term]) -> Result<String, String> {
        match &self.last_check_sat_result {
            Some(r) if r == "sat" => {}
            Some(r) => return Err(format!("get-value: 最近 check-sat 返回 {}, 非 sat", r)),
            None => return Err("get-value: 尚未执行 check-sat".into()),
        }

        // Pass 1: 添加所有查询 term
        let term_ids: Vec<(String, Id)> = terms.iter().map(|t| {
            let display = format_term(t);
            let id = self.add_term(t);
            (display, id)
        }).collect();
        self.rebuild_with_adt();

        // Pass 2: 提取值
        let extractor = Extractor::new(&self.egraph, AstSize);
        let mut result_pairs = Vec::new();
        for (display, id) in &term_ids {
            let (_cost, best_expr) = extractor.find_best(*id);
            result_pairs.push(format!("  ({} {})", display, best_expr));
        }

        Ok(format!("(\n{}\n)", result_pairs.join("\n")))
    }
}


/// 将 AST Term 格式化为 S-expression 字符串（用于 get-value 输出）
fn format_term(term: &Term) -> String {
    match term {
        Term::Var(name) => name.clone(),
        Term::Int(n) => n.to_string(),
        Term::App(f, args) if args.is_empty() => f.clone(),
        Term::App(f, args) => {
            let arg_strs: Vec<String> = args.iter().map(format_term).collect();
            format!("({} {})", f, arg_strs.join(" "))
        }
        Term::Ite(c, t, e) => format!("(ite {} {} {})",
            format_formula(c), format_term(t), format_term(e)),
        Term::Match(s, _) => format!("(match {} ...)", format_term(s)),
        Term::As(inner, sort) => format!("(as {} {})", format_term(inner), sort),
    }
}

/// 将 AST Formula 格式化为 S-expression 字符串
fn format_formula(f: &Formula) -> String {
    match f {
        Formula::True => "true".into(),
        Formula::False => "false".into(),
        Formula::Eq(t1, t2) => format!("(= {} {})", format_term(t1), format_term(t2)),
        Formula::Not(inner) => format!("(not {})", format_formula(inner)),
        Formula::And(fs) => format!("(and {})",
            fs.iter().map(format_formula).collect::<Vec<_>>().join(" ")),
        Formula::Or(fs) => format!("(or {})",
            fs.iter().map(format_formula).collect::<Vec<_>>().join(" ")),
        Formula::Implies(l, r) => format!("(=> {} {})",
            format_formula(l), format_formula(r)),
        Formula::Ite(c, t, e) => format!("(ite {} {} {})",
            format_formula(c), format_formula(t), format_formula(e)),
        Formula::IsTester(c, t) => format!("(is-{} {})", c, format_term(t)),
    }
}

/// 处理 Program 并收集所有 check-sat 结果（供测试使用）
///
/// 返回值: Ok(Vec<String>) — 每次 check-sat 的结果 ("sat" 或 "unsat")
/// 如果处理过程中遇到错误（如类型错误），返回 Err(String)
fn process_program_collect(program: &Program) -> Result<Vec<String>, String> {
    let mut ctx = EggContext::new();
    let mut stack: Vec<EggContext> = Vec::new();
    let mut results: Vec<String> = Vec::new();

    for cmd in &program.commands {
        match cmd {
            Command::SetLogic(_) => {}
            Command::DeclareDatatypes(dts) => {
                for dt in dts {
                    ctx.sort_env.register_datatype(dt);
                }
                ctx.generate_datatype_rules(dts);
            }
            Command::DeclareSort(name, _arity) => {
                if !ctx.known_sorts.contains(name) {
                    ctx.known_sorts.push(name.clone());
                }
                ctx.sort_env.register_sort(name);
            }
            Command::DeclareConst(name, sort_expr) => {
                ctx.sort_env.check_sort_expr(sort_expr, &[])?;
                let sort = sort_expr_to_sort(sort_expr, &[]);
                ctx.sort_env.register_const(name, sort);
                let sort_str = format!("{}", sort_expr);
                ctx.process_declare_const(name, &sort_str);
            }
            Command::DeclareFun(name, arg_sort_exprs, ret_sort_expr) => {
                for se in arg_sort_exprs {
                    ctx.sort_env.check_sort_expr(se, &[])?;
                }
                ctx.sort_env.check_sort_expr(ret_sort_expr, &[])?;
                let arg_sorts: Vec<Sort> = arg_sort_exprs.iter()
                    .map(|se| sort_expr_to_sort(se, &[])).collect();
                let ret_sort = sort_expr_to_sort(ret_sort_expr, &[]);
                if arg_sorts.is_empty() {
                    ctx.sort_env.register_const(name, ret_sort);
                } else {
                    ctx.sort_env.register_fun(name, arg_sorts, ret_sort);
                }
                let arg_strs: Vec<String> = arg_sort_exprs.iter()
                    .map(|se| format!("{}", se)).collect();
                let ret_str = format!("{}", ret_sort_expr);
                ctx.process_declare_fun(name, &arg_strs, &ret_str);
            }
            Command::Assert(formula) => {
                let empty_scope = HashMap::new();
                ctx.sort_env.check_formula(formula, &empty_scope)?;
                ctx.process_assertion(formula);
            }
            Command::CheckSat => {
                let result = ctx.check_sat();
                results.push(result);
            }
            Command::Push(n) => {
                for _ in 0..*n {
                    stack.push(ctx.clone());
                }
            }
            Command::Pop(n) => {
                if stack.len() < *n {
                    return Err(format!("pop {}: 断言栈只有 {} 层", n, stack.len()));
                }
                for _ in 0..(*n - 1) {
                    stack.pop();
                }
                ctx = stack.pop().unwrap();
            }
            Command::GetModel | Command::GetValue(_) => {}
            Command::Exit => break,
            Command::Skip(_) => {}
        }
    }
    Ok(results)
}

/// 处理整个 Program：遍历所有 Command，依次执行
///
/// 执行顺序很重要：
///   1. 先处理 DeclareDatatypes —— 生成 rewrite rules
///   2. 再处理 Assert —— 把 term 加入 e-graph 并处理等式/不等式
///   3. 最后处理 CheckSat —— 运行 saturation 并判断
///
/// 注意：实际 SMT-LIB 规范允许交错使用这些命令，
/// 我们这里按照出现顺序逐一处理，是一个简化但合理的实现。
fn process_program(program: &Program) -> Result<(), String> {
    let mut ctx = EggContext::new();
    let mut stack: Vec<EggContext> = Vec::new(); // push/pop 断言栈

    for cmd in &program.commands {
        match cmd {
            Command::SetLogic(logic) => {
                println!("\n===== set-logic: {} =====", logic);
            }
            Command::DeclareDatatypes(dts) => {
                // 处理数据类型声明：生成 selector + tester 的 rewrite rules
                println!("\n===== 处理 declare-datatypes =====");
                // 注册到 sort 类型环境
                for dt in dts {
                    ctx.sort_env.register_datatype(dt);
                }
                ctx.generate_datatype_rules(dts);
            }
            Command::DeclareSort(name, _arity) => {
                // 注册未解释 sort 名称
                if !ctx.known_sorts.contains(name) {
                    ctx.known_sorts.push(name.clone());
                }
                ctx.sort_env.register_sort(name);
                println!("\n===== declare-sort: {} =====", name);
            }
            Command::DeclareConst(name, sort_expr) => {
                // 处理常量声明：在 e-graph 中注册叶节点
                println!("\n===== 处理 declare-const =====");
                ctx.sort_env.check_sort_expr(sort_expr, &[])?;
                let sort = sort_expr_to_sort(sort_expr, &[]);
                ctx.sort_env.register_const(name, sort);
                let sort_str = format!("{}", sort_expr);
                ctx.process_declare_const(name, &sort_str);
            }
            Command::DeclareFun(name, arg_sort_exprs, ret_sort_expr) => {
                // 处理函数声明：注册函数签名（0元函数同时创建叶节点）
                println!("\n===== 处理 declare-fun =====");
                for se in arg_sort_exprs {
                    ctx.sort_env.check_sort_expr(se, &[])?;
                }
                ctx.sort_env.check_sort_expr(ret_sort_expr, &[])?;
                let arg_sorts: Vec<Sort> = arg_sort_exprs.iter()
                    .map(|se| sort_expr_to_sort(se, &[])).collect();
                let ret_sort = sort_expr_to_sort(ret_sort_expr, &[]);
                if arg_sorts.is_empty() {
                    ctx.sort_env.register_const(name, ret_sort);
                } else {
                    ctx.sort_env.register_fun(name, arg_sorts, ret_sort);
                }
                let arg_strs: Vec<String> = arg_sort_exprs.iter()
                    .map(|se| format!("{}", se)).collect();
                let ret_str = format!("{}", ret_sort_expr);
                ctx.process_declare_fun(name, &arg_strs, &ret_str);
            }
            Command::Assert(formula) => {
                // 类型检查：在 egg 编码之前验证公式类型正确性
                println!("\n===== 处理 assert =====");
                let empty_scope = HashMap::new();
                ctx.sort_env.check_formula(formula, &empty_scope)?;
                println!("[类型检查] assert 公式类型检查通过");
                // 编码到 egg
                ctx.process_assertion(formula);
            }
            Command::CheckSat => {
                // 运行 equality saturation 并检查可满足性
                println!("\n===== 处理 check-sat =====");
                let result = ctx.check_sat();
                println!("\n=============================");
                println!("  check-sat 结果: {}", result);
                println!("=============================");
            }
            Command::Push(n) => {
                println!("\n===== push {} =====", n);
                for _ in 0..*n {
                    stack.push(ctx.clone());
                }
                println!("[push] 断言栈深度: {}", stack.len());
            }
            Command::Pop(n) => {
                println!("\n===== pop {} =====", n);
                if stack.len() < *n {
                    return Err(format!(
                        "pop {}: 断言栈只有 {} 层", n, stack.len()));
                }
                for _ in 0..(*n - 1) {
                    stack.pop();
                }
                ctx = stack.pop().unwrap();
                println!("[pop] 状态已恢复，断言栈深度: {}", stack.len());
            }
            Command::GetModel => {
                println!("\n===== get-model =====");
                match ctx.get_model() {
                    Ok(model) => println!("{}", model),
                    Err(e) => eprintln!("[错误] {}", e),
                }
            }
            Command::GetValue(terms) => {
                println!("\n===== get-value =====");
                match ctx.get_value(terms) {
                    Ok(val) => println!("{}", val),
                    Err(e) => eprintln!("[错误] {}", e),
                }
            }
            Command::Exit => {
                println!("\n===== exit =====");
                break;
            }
            Command::Skip(name) => {
                println!("[跳过] {}", name);
            }
        }
    }

    Ok(())
}


fn main() -> Result<(), String> {
    // ================================================================
    // 测试 1: 类型正确的程序（match + 类型检查 + egg 求解 => unsat）
    // ================================================================
    //
    // 完整流程: parse -> type check -> egg encode -> saturation -> check-sat
    //
    // 推理链:
    //   x = Cons(1, Nil)
    //   match x: pattern (Cons h t) 匹配, h: Int = head(x) = 1
    //   assert (not (= match_result 1)) => 1 != 1 => unsat
    println!("================================================================");
    println!("  测试 1: 类型正确的程序（match + sort 检查 + egg 求解）");
    println!("================================================================\n");

    let input1 = r#"
; 类型正确的 match 测试 — 所有 sort 一致
(set-logic QF_DT)
(declare-datatype List
  ((Nil) (Cons (head Int) (tail List))))
(declare-const x List)
(assert (= x (Cons 1 Nil)))
; match 在 term 位置: 对 x 模式匹配，提取 head
(assert (not (= (match x (Nil 0) ((Cons h t) h)) 1)))
(check-sat)
(exit)
    "#;

    let program1 = parse_program(input1)?;
    process_program(&program1)?;

    // ================================================================
    // 测试 2: 类型错误检测 — (= x n) 其中 x: List, n: Int
    // ================================================================
    //
    // 期望: 类型检查阶段报错，不进入 egg 编码
    println!("\n\n================================================================");
    println!("  测试 2: 类型错误检测 — (= x n)，x: List, n: Int");
    println!("================================================================\n");

    let input2 = r#"
(set-logic QF_DT)
(declare-datatype List
  ((Nil) (Cons (head Int) (tail List))))
(declare-const x List)
(declare-const n Int)
; 类型错误: x 是 List，n 是 Int，不能比较等式
(assert (= x n))
(check-sat)
    "#;

    let program2 = parse_program(input2)?;
    match process_program(&program2) {
        Ok(()) => println!("[测试 2] 未检测到错误（非预期）"),
        Err(e) => println!("[测试 2] 类型错误被成功捕获: {}", e),
    }

    // ================================================================
    // 测试 3: 类型错误检测 — 构造器参数 sort 错误
    // ================================================================
    //
    // (Cons Nil Nil): 第一个参数期望 Int，实际是 List
    println!("\n\n================================================================");
    println!("  测试 3: 类型错误检测 — 构造器参数 sort 不匹配");
    println!("================================================================\n");

    let input3 = r#"
(set-logic QF_DT)
(declare-datatype List
  ((Nil) (Cons (head Int) (tail List))))
(declare-const x List)
; 类型错误: Cons 的第一个参数期望 Int，给了 Nil (List)
(assert (= x (Cons Nil Nil)))
(check-sat)
    "#;

    let program3 = parse_program(input3)?;
    match process_program(&program3) {
        Ok(()) => println!("[测试 3] 未检测到错误（非预期）"),
        Err(e) => println!("[测试 3] 类型错误被成功捕获: {}", e),
    }

    // ================================================================
    // 测试 4: 参数化数据类型 + as 消歧 — 类型正确（unsat）
    // ================================================================
    //
    // 使用 par 声明参数化 List，用 (as nil (List Int)) 消歧多态构造器
    // 推理链:
    //   xs = (as nil (List Int))
    //   (cons 1 xs) = (cons 1 nil)  [经 rewrite]
    //   head((cons 1 nil)) => 1
    //   assert (not (= 1 1)) => unsat
    println!("\n\n================================================================");
    println!("  测试 4: 参数化数据类型 + as 消歧（par + unsat）");
    println!("================================================================\n");

    let input4 = r#"
(set-logic QF_DT)
; 参数化 List 声明: List 是 arity=1 的数据类型，类型参数 T
(declare-datatypes ((List 1))
  ((par (T) ((nil) (cons (hd T) (tl (List T)))))))
; 声明常量: xs 是 (List Int)
(declare-const xs (List Int))
; xs = nil (消歧为 (List Int))
(assert (= xs (as nil (List Int))))
; head(cons(1, xs)) 应该是 1
(assert (not (= (hd (cons 1 xs)) 1)))
(check-sat)
(exit)
    "#;

    let program4 = parse_program(input4)?;
    process_program(&program4)?;

    // ================================================================
    // 测试 5: 参数化数据类型 — 类型错误检测
    // ================================================================
    //
    // (= xs ys) 其中 xs: (List Int), ys: (List Bool) — sort 不匹配
    println!("\n\n================================================================");
    println!("  测试 5: 参数化类型错误 — (List Int) vs (List Bool)");
    println!("================================================================\n");

    let input5 = r#"
(set-logic QF_DT)
(declare-datatypes ((List 1))
  ((par (T) ((nil) (cons (hd T) (tl (List T)))))))
(declare-const xs (List Int))
(declare-const ys (List Bool))
; 类型错误: xs: (List Int), ys: (List Bool)，不能比较等式
(assert (= xs ys))
(check-sat)
    "#;

    let program5 = parse_program(input5)?;
    match process_program(&program5) {
        Ok(()) => println!("[测试 5] 未检测到错误（非预期）"),
        Err(e) => println!("[测试 5] 类型错误被成功捕获: {}", e),
    }

    // ================================================================
    // 测试 6: push/pop 断言栈 + get-model
    // ================================================================
    //
    // 流程:
    //   1. 声明 x: List，断言 x = Cons(1, Nil)
    //   2. check-sat => sat, get-model 输出 x 的值
    //   3. push 保存状态
    //   4. 追加矛盾断言 x = Nil，check-sat => unsat
    //   5. pop 恢复状态
    //   6. check-sat => sat（矛盾断言被撤销），get-model 再次输出
    println!("\n\n================================================================");
    println!("  测试 6: push/pop 断言栈 + get-model");
    println!("================================================================\n");

    let input6 = r#"
(set-logic QF_DT)
(declare-datatype List
  ((Nil) (Cons (head Int) (tail List))))
(declare-const x List)
(assert (= x (Cons 1 Nil)))
; 第一次 check-sat: sat（x 有一致的赋值）
(check-sat)
(get-model)
; push 保存当前状态
(push 1)
; 追加矛盾断言: x 同时是 Nil
(assert (= x Nil))
; check-sat: unsat（Cons(1,Nil) != Nil）
(check-sat)
; pop 恢复到 push 之前的状态
(pop 1)
; 再次 check-sat: sat（矛盾断言已被撤销）
(check-sat)
(get-model)
(exit)
    "#;

    let program6 = parse_program(input6)?;
    process_program(&program6)?;

    // ================================================================
    // 测试 7: get-value 查询 + push/pop 组合
    // ================================================================
    //
    // 查询特定 term 在模型中的值
    println!("\n\n================================================================");
    println!("  测试 7: get-value 查询特定 term 的值");
    println!("================================================================\n");

    let input7 = r#"
(set-logic QF_DT)
(declare-datatype List
  ((Nil) (Cons (head Int) (tail List))))
(declare-const x List)
(declare-const y Int)
(assert (= x (Cons 42 Nil)))
(assert (= y (head x)))
(check-sat)
; 查询 x, y, (head x), (tail x) 的值
(get-value (x y (head x) (tail x)))
(exit)
    "#;

    let program7 = parse_program(input7)?;
    process_program(&program7)?;

    // ================================================================
    // 测试 8: 多层 push/pop 嵌套
    // ================================================================
    //
    // 验证多层嵌套的正确性: push 2 层, 中间分别添加断言, pop 回到最初
    println!("\n\n================================================================");
    println!("  测试 8: 多层 push/pop 嵌套");
    println!("================================================================\n");

    let input8 = r#"
(set-logic QF_DT)
(declare-datatype Color ((Red) (Green) (Blue)))
(declare-const c Color)
(assert (= c Red))
(check-sat)
(get-model)
; 进入第一层
(push 1)
(assert (not (= c Red)))
; Red 且非 Red => unsat
(check-sat)
; 进入第二层（在 unsat 状态下）
(push 1)
; pop 2 层回到最初
(pop 2)
; 回到只有 c = Red 的状态
(check-sat)
(get-model)
(exit)
    "#;

    let program8 = parse_program(input8)?;
    process_program(&program8)?;

    Ok(())
}

// ============================================================================
// 测试模块: 验证 SMT-LIB (QF_ADT) → egg 转换的完整性和正确性
// ============================================================================
//
// 运行方式: cargo test
// 运行单个分类: cargo test test_selector    (运行所有名字含 "test_selector" 的测试)
// 显示输出:   cargo test -- --nocapture
//
// 测试设计原则:
//   每个测试对应 QF_ADT 理论的一个具体公理或语义性质。
//   测试通过 process_program_collect 执行完整 pipeline:
//     SMT-LIB 文本 → 词法分析 → S-expression → AST → 类型检查 → egg 编码 → 等式饱和 → 判定
//   最终对 check-sat 结果做 assert，确保结果与理论预期一致。
//
// 测试覆盖的 QF_ADT 公理:
//   1. Selector-Constructor (选择子公理): sel_i(C(x1,...,xn)) = x_i
//   2. Constructor Disjointness (构造子互斥): C_i(...) ≠ C_j(...) when i ≠ j
//   3. Tester Semantics (测试子语义): is-C(C(...)) = true, is-C(D(...)) = false
//   4. Constructor Injectivity (构造子单射): C(a,b) = C(c,d) → a=c ∧ b=d
//   5. Match Desugaring (模式匹配脱糖): match 正确转化为 ite + selector
//   6. Parametric ADT (参数化数据类型): par + as 消歧
//   7. Type Checking (类型检查): 拒绝类型不一致的程序
//   8. Push/Pop (断言栈): 正确的状态保存与恢复
//   9. Boolean Reasoning (布尔推理): egg 内建布尔化简规则
//   10. 复合推理: 多个公理协同工作
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数: 运行 SMT-LIB 脚本，返回所有 check-sat 结果
    fn run(input: &str) -> Result<Vec<String>, String> {
        let program = parse_program(input)?;
        process_program_collect(&program)
    }

    /// 辅助函数: 运行脚本并断言恰好有一个 check-sat 结果
    fn run_expect_one(input: &str) -> Result<String, String> {
        let results = run(input)?;
        assert_eq!(results.len(), 1, "期望恰好 1 个 check-sat 结果，实际有 {}", results.len());
        Ok(results.into_iter().next().unwrap())
    }

    // ====================================================================
    // 1. Selector-Constructor 公理: sel_i(C(x1,...,xn)) = x_i
    // ====================================================================

    #[test]
    fn test_selector_head_of_cons() {
        // head(Cons(42, Nil)) = 42 应该可满足
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (= (head (Cons 42 Nil)) 42))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "head(Cons(42, Nil)) = 42 应为 sat");
    }

    #[test]
    fn test_selector_head_of_cons_unsat() {
        // head(Cons(42, Nil)) ≠ 42 应该不可满足
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (not (= (head (Cons 42 Nil)) 42)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "head(Cons(42, Nil)) ≠ 42 应为 unsat");
    }

    #[test]
    fn test_selector_tail_of_cons() {
        // tail(Cons(1, Nil)) = Nil 应该可满足
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (= (tail (Cons 1 Nil)) Nil))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "tail(Cons(1, Nil)) = Nil 应为 sat");
    }

    #[test]
    fn test_selector_tail_not_nil_unsat() {
        // tail(Cons(1, Nil)) ≠ Nil 应该不可满足
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (not (= (tail (Cons 1 Nil)) Nil)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "tail(Cons(1, Nil)) ≠ Nil 应为 unsat");
    }

    #[test]
    fn test_selector_chained() {
        // head(tail(Cons(1, Cons(2, Nil)))) = 2
        // 需要两步 rewrite: tail(Cons(1, Cons(2, Nil))) => Cons(2, Nil)
        //                   head(Cons(2, Nil)) => 2
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (not (= (head (tail (Cons 1 (Cons 2 Nil)))) 2)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "head(tail(Cons(1, Cons(2, Nil)))) ≠ 2 应为 unsat");
    }

    // ====================================================================
    // 2. Constructor Disjointness 公理: C_i ≠ C_j
    // ====================================================================

    #[test]
    fn test_disjoint_nil_cons() {
        // Nil ≠ Cons(1, Nil) — 不同构造子产生不同值
        // 这通过 tester 规则间接验证:
        // is-Nil(Nil)=true, is-Nil(Cons(...))=false,
        // 若 Nil = Cons(...) 则 true = false → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x Nil))
            (assert (= x (Cons 1 Nil)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "x = Nil ∧ x = Cons(1, Nil) 应为 unsat (构造子互斥)");
    }

    #[test]
    fn test_disjoint_enum_constructors() {
        // 枚举类型: 三个不同构造子
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Color ((Red) (Green) (Blue)))
            (declare-const c Color)
            (assert (= c Red))
            (assert (= c Green))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "c = Red ∧ c = Green 应为 unsat (枚举构造子互斥)");
    }

    #[test]
    fn test_disjoint_three_way() {
        // 同时断言三种不同构造子
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Color ((Red) (Green) (Blue)))
            (declare-const c Color)
            (assert (= c Red))
            (assert (not (= c Green)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "c = Red ∧ c ≠ Green 应为 sat");
    }

    // ====================================================================
    // 3. Tester 语义: is-C(C(...)) = true, is-C(D(...)) = false
    // ====================================================================

    #[test]
    fn test_tester_positive_nullary() {
        // is-Nil(Nil) = true
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (not (is-Nil Nil)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "¬is-Nil(Nil) 应为 unsat (正向 tester)");
    }

    #[test]
    fn test_tester_positive_nary() {
        // is-Cons(Cons(1, Nil)) = true
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (not (is-Cons (Cons 1 Nil))))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "¬is-Cons(Cons(1,Nil)) 应为 unsat (正向 tester)");
    }

    #[test]
    fn test_tester_negative() {
        // is-Nil(Cons(1, Nil)) = false
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (is-Nil (Cons 1 Nil)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "is-Nil(Cons(1,Nil)) 应为 unsat (反向 tester)");
    }

    #[test]
    fn test_tester_negative_reverse() {
        // is-Cons(Nil) = false
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (assert (is-Cons Nil))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "is-Cons(Nil) 应为 unsat (反向 tester)");
    }

    #[test]
    fn test_tester_with_variable() {
        // x = Cons(1, Nil) → is-Cons(x) 应为 true
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (assert (not (is-Cons x)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "x = Cons(1,Nil) ∧ ¬is-Cons(x) 应为 unsat");
    }

    // ====================================================================
    // 4. Constructor Injectivity: C(a,b) = C(c,d) → a=c ∧ b=d
    // ====================================================================

    #[test]
    fn test_injectivity_via_selector() {
        // Cons(a, Nil) = Cons(b, Nil) 且 a ≠ b → unsat
        // 经过 selector rewrite:
        //   head(Cons(a, Nil)) = a, head(Cons(b, Nil)) = b
        //   Cons(a, Nil) = Cons(b, Nil) → head 相同 → a = b
        //   但 a ≠ b → 矛盾
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const a Int)
            (declare-const b Int)
            (assert (= (Cons a Nil) (Cons b Nil)))
            (assert (not (= a b)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "Cons(a,Nil)=Cons(b,Nil) ∧ a≠b 应为 unsat (单射性)");
    }

    #[test]
    fn test_injectivity_second_field() {
        // Cons(1, xs) = Cons(1, ys) 且 xs ≠ ys → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const xs List)
            (declare-const ys List)
            (assert (= (Cons 1 xs) (Cons 1 ys)))
            (assert (not (= xs ys)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "Cons(1,xs)=Cons(1,ys) ∧ xs≠ys 应为 unsat (第二字段单射)");
    }

    // ====================================================================
    // 5. Match 表达式脱糖
    // ====================================================================

    #[test]
    fn test_match_basic() {
        // match x { Nil => 0, Cons(h,t) => h } 当 x = Cons(1, Nil)
        // 应该得到 1，断言 ≠ 1 → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (assert (not (= (match x (Nil 0) ((Cons h t) h)) 1)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "match Cons(1,Nil) 取 head 应得 1");
    }

    #[test]
    fn test_match_nil_branch() {
        // match x { Nil => 0, Cons(h,t) => h } 当 x = Nil
        // 应该得到 0，断言 ≠ 0 → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x Nil))
            (assert (not (= (match x (Nil 0) ((Cons h t) h)) 0)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "match Nil 应走 Nil 分支得 0");
    }

    #[test]
    fn test_match_sat() {
        // match 结果 = 1 且 x = Cons(1, Nil): 一致 → sat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (assert (= (match x (Nil 0) ((Cons h t) h)) 1))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "match 结果 = 1 且 x = Cons(1,Nil) 应为 sat");
    }

    // ====================================================================
    // 6. 参数化数据类型 (par + as)
    // ====================================================================

    #[test]
    fn test_parametric_selector() {
        // 参数化 List: hd(cons(1, nil)) = 1
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatypes ((List 1))
              ((par (T) ((nil) (cons (hd T) (tl (List T)))))))
            (declare-const xs (List Int))
            (assert (= xs (as nil (List Int))))
            (assert (not (= (hd (cons 1 xs)) 1)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "hd(cons(1, nil)) ≠ 1 应为 unsat (参数化 ADT)");
    }

    #[test]
    fn test_parametric_sat() {
        // 参数化 List: hd(cons(1, nil)) = 1 — sat 方向
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatypes ((List 1))
              ((par (T) ((nil) (cons (hd T) (tl (List T)))))))
            (declare-const xs (List Int))
            (assert (= xs (cons 1 (as nil (List Int)))))
            (assert (= (hd xs) 1))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "hd(cons(1, nil)) = 1 应为 sat (参数化 ADT)");
    }

    // ====================================================================
    // 7. 类型检查 — 拒绝类型不一致的程序
    // ====================================================================

    #[test]
    fn test_type_error_sort_mismatch() {
        // x: List, n: Int, (= x n) 应被类型检查拒绝
        let result = run(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (declare-const n Int)
            (assert (= x n))
            (check-sat)
        "#);
        assert!(result.is_err(), "(= x:List n:Int) 应触发类型错误");
    }

    #[test]
    fn test_type_error_ctor_arg() {
        // Cons 第一个参数期望 Int，给了 Nil (List)
        let result = run(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons Nil Nil)))
            (check-sat)
        "#);
        assert!(result.is_err(), "Cons(Nil, Nil) 应触发类型错误: 第一参数类型不匹配");
    }

    #[test]
    fn test_type_error_parametric_mismatch() {
        // xs: (List Int), ys: (List Bool), (= xs ys) 类型不匹配
        let result = run(r#"
            (set-logic QF_DT)
            (declare-datatypes ((List 1))
              ((par (T) ((nil) (cons (hd T) (tl (List T)))))))
            (declare-const xs (List Int))
            (declare-const ys (List Bool))
            (assert (= xs ys))
            (check-sat)
        "#);
        assert!(result.is_err(), "(= xs:(List Int) ys:(List Bool)) 应触发类型错误");
    }

    // ====================================================================
    // 8. Push/Pop 断言栈
    // ====================================================================

    #[test]
    fn test_push_pop_basic() {
        // sat → push → 追加矛盾 → unsat → pop → sat
        let results = run(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (check-sat)
            (push 1)
            (assert (= x Nil))
            (check-sat)
            (pop 1)
            (check-sat)
        "#).unwrap();
        assert_eq!(results, vec!["sat", "unsat", "sat"],
            "push/pop 应正确保存和恢复状态");
    }

    #[test]
    fn test_push_pop_nested() {
        // 多层 push/pop
        let results = run(r#"
            (set-logic QF_DT)
            (declare-datatype Color ((Red) (Green) (Blue)))
            (declare-const c Color)
            (assert (= c Red))
            (check-sat)
            (push 1)
            (assert (not (= c Red)))
            (check-sat)
            (push 1)
            (pop 2)
            (check-sat)
        "#).unwrap();
        assert_eq!(results, vec!["sat", "unsat", "sat"],
            "多层 push/pop 应正确恢复");
    }

    // ====================================================================
    // 9. 布尔推理
    // ====================================================================

    #[test]
    fn test_boolean_not_not() {
        // not(not(= x Nil)) 等价于 (= x Nil)
        // 断言 x = Nil 且 not(not(x = Nil)) 为假 → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x Nil))
            (assert (not (not (not (= x Nil)))))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "x=Nil ∧ ¬¬¬(x=Nil) 应为 unsat (双重否定消除)");
    }

    #[test]
    fn test_boolean_and_propagation() {
        // and(x=Nil, x=Cons(1,Nil)) → 两个都为 true → x 同时是 Nil 和 Cons → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (and (= x Nil) (= x (Cons 1 Nil))))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "x=Nil ∧ x=Cons(1,Nil) 应为 unsat (and + 构造子互斥)");
    }

    #[test]
    fn test_boolean_implies() {
        // x = Cons(1, Nil) → head(x) = 1 应恒成立
        // 断言蕴含为假 → unsat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (assert (not (=> (= x (Cons 1 Nil)) (= (head x) 1))))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "x=Cons(1,Nil) → head(x)=1 的否定应为 unsat");
    }

    // ====================================================================
    // 10. 复合推理: 多个公理协同工作
    // ====================================================================

    #[test]
    fn test_composite_selector_plus_disjoint() {
        // x = Cons(1, Nil)
        // head(x) = 1 (selector) 且 x ≠ Nil (disjoint) 且 is-Cons(x) (tester)
        // 全部一致 → sat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 1 Nil)))
            (assert (= (head x) 1))
            (assert (not (= x Nil)))
            (assert (is-Cons x))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "selector + disjoint + tester 综合应为 sat");
    }

    #[test]
    fn test_composite_multi_constructor_adt() {
        // 多构造子 ADT: Tree = Leaf(Int) | Node(Tree, Tree)
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Tree
              ((Leaf (val Int))
               (Node (left Tree) (right Tree))))
            (declare-const t Tree)
            (assert (= t (Node (Leaf 1) (Leaf 2))))
            (assert (not (= (val (left t)) 1)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "val(left(Node(Leaf(1), Leaf(2)))) ≠ 1 应为 unsat");
    }

    #[test]
    fn test_composite_tree_right_branch() {
        // val(right(Node(Leaf(1), Leaf(2)))) = 2
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Tree
              ((Leaf (val Int))
               (Node (left Tree) (right Tree))))
            (declare-const t Tree)
            (assert (= t (Node (Leaf 1) (Leaf 2))))
            (assert (not (= (val (right t)) 2)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "val(right(Node(Leaf(1),Leaf(2)))) ≠ 2 应为 unsat");
    }

    #[test]
    fn test_composite_enum_exhaustive() {
        // 枚举类型 Day，验证多重不等式
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Day
              ((Mon) (Tue) (Wed) (Thu) (Fri) (Sat) (Sun)))
            (declare-const d Day)
            (assert (= d Mon))
            (assert (not (= d Tue)))
            (assert (not (= d Wed)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "d=Mon ∧ d≠Tue ∧ d≠Wed 应为 sat");
    }

    #[test]
    fn test_composite_pair_adt() {
        // Pair ADT with two fields
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Pair
              ((MkPair (fst Int) (snd Int))))
            (declare-const p Pair)
            (assert (= p (MkPair 3 4)))
            (assert (not (= (fst p) 3)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "fst(MkPair(3,4)) ≠ 3 应为 unsat");
    }

    #[test]
    fn test_composite_ite_term() {
        // term-level ite: (ite (is-Nil x) 0 (head x))
        // x = Cons(5, Nil) → is-Nil(x) = false → ite 选 else 分支 → head(x) = 5
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x (Cons 5 Nil)))
            (assert (not (= (ite (is-Nil x) 0 (head x)) 5)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "ite(is-Nil(Cons(5,Nil)), 0, head(Cons(5,Nil))) ≠ 5 应为 unsat");
    }

    // ====================================================================
    // 11. 边界情况
    // ====================================================================

    #[test]
    fn test_trivial_sat() {
        // 空断言集 → sat
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Unit ((MkUnit)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "sat", "空断言集应为 sat");
    }

    #[test]
    fn test_trivial_unsat_false() {
        // 直接断言 false
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Unit ((MkUnit)))
            (assert (not (= MkUnit MkUnit)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "断言 MkUnit ≠ MkUnit 应为 unsat");
    }

    #[test]
    fn test_single_constructor_adt() {
        // 只有一个构造子的 ADT
        let r = run_expect_one(r#"
            (set-logic QF_DT)
            (declare-datatype Wrapper ((Wrap (unwrap Int))))
            (declare-const w Wrapper)
            (assert (= w (Wrap 10)))
            (assert (not (= (unwrap w) 10)))
            (check-sat)
        "#).unwrap();
        assert_eq!(r, "unsat", "unwrap(Wrap(10)) ≠ 10 应为 unsat");
    }

    #[test]
    fn test_multiple_check_sat() {
        // 多次 check-sat（不用 push/pop）
        let results = run(r#"
            (set-logic QF_DT)
            (declare-datatype List ((Nil) (Cons (head Int) (tail List))))
            (declare-const x List)
            (assert (= x Nil))
            (check-sat)
            (assert (= x (Cons 1 Nil)))
            (check-sat)
        "#).unwrap();
        assert_eq!(results[0], "sat", "第一次 check-sat: x=Nil 应为 sat");
        assert_eq!(results[1], "unsat", "第二次 check-sat: x=Nil ∧ x=Cons(1,Nil) 应为 unsat");
    }
}