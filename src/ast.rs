/***
 * 演算子を表現する列挙体
 */
#[derive(Debug, Clone)]
enum OperatorKind {
    Add,
    Sub,
    Mul,
    Div,
}
/***
 * 式（Expression）を定義・表現する列挙体
 */
#[derive(Debug, Clone)]
enum Expr {
    Operation(Box<BinaryOp>),
    Number(Option<Number>),
    Operator(OperatorKind),
}
/***
 * 2項演算を表現する構造体
 */
#[derive(Debug, Clone)]
struct BinaryOp {
    l: Expr,
    r: Expr,
    op: OperatorKind,
}
/***
 * 2項演算を構成する要素（2つの数値と演算子）を受け取り式を返す
 */
impl BinaryOp {
    fn new(l: Expr, r: Expr, op: OperatorKind) -> Expr {
        Expr::Operation(Box::new(BinaryOp { l: l, r: r, op: op }))
    }
}

/***
 * 数値を表現する構造体
 */
#[derive(Debug, Clone)]
struct Number(u128);
impl Number {
    fn new(n: u128) -> Expr {
        Expr::Number(Some(Number(n)))
    }
    fn from_expr(expr: Expr) -> Option<Number> {
        match expr {
            Expr::Number(number) => number,
            _ => None,
        }
    }
}
impl std::ops::Sub for Number {
    type Output = Expr;
    fn sub(self, other: Self) -> Self::Output {
        Number::new(self.0 - other.0)
    }
}
impl std::ops::Add for Number {
    type Output = Expr;
    fn add(self, other: Self) -> Self::Output {
        Number::new(self.0 + other.0)
    }
}
impl std::ops::Mul for Number {
    type Output = Expr;
    fn mul(self, other: Self) -> Self::Output {
        Number::new(self.0 * other.0)
    }
}
impl std::ops::Div for Number {
    type Output = Expr;
    fn div(self, other: Self) -> Self::Output {
        Number::new(self.0 / other.0)
    }
}

impl std::cmp::PartialEq<u128> for Number {
    fn eq(&self, rhs: &u128) -> bool {
        self.0 == *rhs
    }
}

/***
 * 算術エラーを表現する列挙型
 * Success:成功（エラーなし、解あり）
 * DivByZero: 零除算（算術エラー、解無し）
 */
enum ArithmeticErrorKind {
    Success,
    DivByZero,
}

#[derive(Debug)]
struct ArithmeticError {
    e_code: usize,
}
impl ArithmeticError {
    fn new(code: ArithmeticErrorKind) -> Self {
        ArithmeticError {
            e_code: code as usize,
        }
    }
    fn to_enum(&self) -> Option<ArithmeticErrorKind> {
        let success = ArithmeticErrorKind::Success as usize;
        let div_by_zero = ArithmeticErrorKind::DivByZero as usize;
        match self.e_code {
            success => Some(ArithmeticErrorKind::Success),
            div_by_zero => Some(ArithmeticErrorKind::DivByZero),
            _ => None,
        }
    }
    // コンフィグから呼び出す感じにしたい。手抜き
    fn resolve_string(&self) -> String {
        match self.to_enum() {
            Some(ArithmeticErrorKind::Success) => "成功しました。",
            Some(ArithmeticErrorKind::DivByZero) => "解無し：ゼロ除算が発生しました。",
            None => "存在しないエラーコードが指定されました。",
        }
        .to_owned()
    }
}
impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.resolve_string())
    }
}
impl std::error::Error for ArithmeticError {}

/***
 * Ok(Number):計算結果
 * Err(ArithmeticError):解無し(0除算など数学的に解が出ないもの)
 */
fn parser(op: Expr) -> Result<Number, ArithmeticError> {
    // 普通の式
    let mut parser_stack = Vec::new();
    parser_stack.push(op);
    // 式を分解して、逆ポーランド記法的な感じでスタックに突っ込んでいく。
    // 逆ポーランド記法で入れていくスタック
    let mut rev_polish = Vec::new();
    loop {
        let v = parser_stack.pop();
        match v.unwrap() {
            Expr::Number(num) => {
                // スタックが空になったらパースを終了する。
                rev_polish.push(Expr::Number(num));
                if parser_stack.len() == 0 {
                    break;
                }
            }
            Expr::Operation(op) => {
                // 式を分解して、逆ポーランド記法的な雰囲気でスタックに突っ込んでいく。
                rev_polish.push(Expr::Operator(op.op));
                parser_stack.push(op.l);
                parser_stack.push(op.r);
            }
            _ => panic!(), //ここには絶対に来ない。来たら死ぬ。
        }
    }
    // スタックに逆ポーランド的に構成されているので粛々と計算する。
    // 回答用スタックを用意する
    let mut ans = Vec::new();
    loop {
        match rev_polish.pop().unwrap() {
            // 何かしら数値が入っていれば、途中経過の計算結果として扱う。
            Expr::Number(num) => {
                ans.push(num);
                // 逆ポーランド記法スタックが空、かつ回答用スタックに1個であれば計算終了とする
                if rev_polish.len() == 0 && ans.len() == 1 {
                    break;
                }
            }
            // 途中経過の計算結果から2個回答を取り出して計算を行う。
            Expr::Operator(op) => {
                let r = ans.pop().unwrap().unwrap();
                let l = ans.pop().unwrap().unwrap();
                let ans = match op {
                    OperatorKind::Add => l + r,
                    OperatorKind::Sub => l - r,
                    OperatorKind::Mul => l * r,
                    OperatorKind::Div => {
                        if r != 0 {
                            l / r
                        } else {
                            return Err(ArithmeticError::new(ArithmeticErrorKind::DivByZero));
                        }
                    }
                };
                rev_polish.push(ans);
            }
            _ => panic!(), // ここには来ない。
        }
    }
    // スタックが0で取得できないことはありえない（解無し）という前提がある。
    // さらに、解なしの場合はOption<Number>もunwrap出来ないので正常にリターンできないはず。
    // むしろそれが起きたらpanicするのが正しいのでunwrapの実装でOK。
    Ok(ans.pop().unwrap().unwrap())
}

#[test]
fn parser_test1() {
    // 14
    let a = Number::new(14);
    assert_eq!(parser(a).unwrap(), 14);
}

#[test]
fn parser_test2() {
    // 14 - 2
    let a = BinaryOp::new(Number::new(14), Number::new(2), OperatorKind::Sub);
    assert_eq!(parser(a).unwrap(), 12);
}

#[test]
fn parser_test3() {
    // 10 + 2
    let a = BinaryOp::new(Number::new(14), Number::new(2), OperatorKind::Sub);
    let b = Number::new(2);
    // ((14-2)*2)
    let c = BinaryOp::new(a, b, OperatorKind::Mul);
    assert_eq!(parser(c).unwrap(), 24);
}

#[test]
fn parser_test4() {
    // 10 + 2
    let a = Number::new(2);
    let b = BinaryOp::new(Number::new(14), Number::new(2), OperatorKind::Sub);
    // (2*(14-2))
    let c = BinaryOp::new(a, b, OperatorKind::Mul);
    assert_eq!(parser(c).unwrap(), 24);
}

#[test]
fn parser_test5() {
    // 10 + 2
    let a = BinaryOp::new(Number::new(14), Number::new(2), OperatorKind::Sub);
    let b = Number::new(2);
    // ((10+2)*2)
    let c = BinaryOp::new(a, b, OperatorKind::Mul);
    // (((14-2)*2))+(((14-2)*2))
    let d = BinaryOp::new(c.clone(), c, OperatorKind::Add);
    assert_eq!(parser(d.clone()).unwrap(), 48);
}
