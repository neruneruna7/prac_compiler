// EBNFからFIRST集合を求めるプログラム
// []は1回または0回
// {}は0回以上の繰り返し
// |はまたは
// ->は導出
// ::=は定義
// EBNFの例
// <A> ::= <B> | <C> | <D>
// <B> ::= "b"
// <C> ::= [ "c" ] "b"
// <D> ::= { "d" } "b"
use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex, OnceLock},
};

#[derive(Debug, Clone)]
struct Rule {
    left: String,
    right: String,
}

type FirstSets = HashMap<Vec<String>, HashSet<String>>;

// Once_cellで1度だけ書き換え可能な文法規則のベクタ
static RULES: OnceLock<Vec<Rule>> = OnceLock::new();

// FIRST集合計算結果のキャッシュ
static COMPLETED_FIRST_SET: LazyLock<Mutex<FirstSets>> =
    LazyLock::new(|| Mutex::new(FirstSets::new()));

fn main() {
    let rules = [
        Rule {
            left: "Program".to_string(),
            right: "program name ; Block .".to_string(),
        },
        Rule {
            left: "Block".to_string(),
            right: "{ VarDec | SubDec } ComSta".to_string(),
        },
        Rule {
            left: "VarDec".to_string(),
            right: "var VarNames : Type ; { VarNames : Type ; } ".to_string(),
        },
        Rule {
            left: "VarNames".to_string(),
            right: "VarName { , VarName }".to_string(),
        },
        Rule {
            left: "VarName".to_string(),
            right: "name".to_string(),
        },
        Rule {
            left: "Type".to_string(),
            right: "StdType | ArrType".to_string(),
        },
        Rule {
            left: "StdType".to_string(),
            right: "integer | boolean | char".to_string(),
        },
        Rule {
            left: "ArrType".to_string(),
            right: "array '[' uint ']' of StdType".to_string(),
        },
        Rule {
            left: "SubDec".to_string(),
            right: "procedure ProcName [ ForPar ] ; [ VarDec ] ComSta".to_string(),
        },
        Rule {
            left: "ProcName".to_string(),
            right: "name".to_string(),
        },
        Rule {
            left: "ForPar".to_string(),
            right: "'(' VarNames : Type { ; VarNames : Type } ')' ".to_string(),
        },
        Rule {
            left: "ComSta".to_string(),
            right: "begin Sta { ; Sta } end".to_string(),
        },
        Rule {
            left: "Sta".to_string(),
            right: "AssSta | CndSta | ItrSta | ExtSta | CalSta | RetSta | InpSta | OutSta | ComSta | EmpSta".to_string(),
        },
        Rule {
            left: "CndSta".to_string(),
            right: "if Exp then Sta [ else Sta ]".to_string(),
        },
        Rule {
            left: "ItrSta".to_string(),
            right: "while Exp do Sta".to_string(),
        },
        Rule {
            left: "ExtSta".to_string(),
            right: "break".to_string(),
        },
        Rule {
            left: "CalSta".to_string(),
            right: "call ProcName [ '(' Exps ')' ]".to_string(),
        },
        Rule {
            left: "Exps".to_string(),
            right: "Exp { , Exp }".to_string(),
        },
        Rule {
            left: "RetSta".to_string(),
            right: "return".to_string(),
        },
        Rule
        {
            left: "AssSta".to_string(),
            right: "LeftPart := Exp".to_string(),
        },
        Rule {
            left: "LeftPart".to_string(),
            right: "Var".to_string(),
        },
        Rule {
            left: "Var".to_string(),
            right: "name [ '[' Exp ']' ]".to_string(),
        },
        Rule {
            left: "Exp".to_string(),
            right: "SimpleExp { RelaOpe SimpleExp }".to_string(),
        },
        Rule {
            left: "SimpleExp".to_string(),
            right: "[ + | - ] Term { AddOpe Term }".to_string(),
        },
        Rule {
            left: "Term".to_string(),
            right: "Factor { MulOpe Factor }".to_string(),
        },
        Rule {
            left: "Factor".to_string(),
            right: "Var | Cons | '(' Exp ')' | not Factor | StdType '(' Exp ')'".to_string(),
        },
        Rule {
            left: "Cons".to_string(),
            right: "uint | false | true | string".to_string(),
        },
        Rule {
            left: "MulOpe".to_string(),
            right: "* | div | and".to_string(),
        },
        Rule {
            left: "AddOpe".to_string(),
            right: "+ | - | or".to_string(),
        },
        Rule {
            left: "RelaOpe".to_string(),
            right: "= | <> | < | <= | > | >=".to_string(),
        },
        Rule {
            left: "InpSta".to_string(),
            right: "( read | readln ) { '(' Var { , Var } ')' }".to_string(),
        },
        Rule {
            left: "OutSta".to_string(),
            right: "( write | writeln ) { '(' OutFor { , OutFor } ')' }".to_string(),
        },
        Rule {
            left: "OutFor".to_string(),
            right: "Exp [ : uint ] | string".to_string(),
        },
        Rule {
            left: "EmpSta".to_string(),
            right: "ε".to_string(),
        },
    ];

    let first_set = calc_first_set(rules.to_vec());
    for (k, v) in first_set.iter() {
        println!("FIRST({}) = {:?}", k, v);
    }
    // println!("{:?}", COMPLETED_FIRST_SET.lock().unwrap());
}

fn calc_first_set(rules: Vec<Rule>) -> HashMap<String, HashSet<String>> {
    // rulesをOnce_cellでグローバル変数に代入する
    RULES.get_or_init(|| rules);

    let mut first_set = HashMap::new();
    // 文法規則すべてについて，FIRST集合を求める
    for r in RULES.get().unwrap().iter() {
        // 空白で区切る
        let left = r.left.split_whitespace().collect::<Vec<&str>>();

        let set = first(&left);
        first_set.insert(r.left.clone(), set);
    }

    first_set
}

fn first(a: &[&str]) -> HashSet<String> {
    // FIRST(a)が計算済みなら，その値を返す
    if let Some(set) = COMPLETED_FIRST_SET
        .lock()
        .unwrap()
        .get(&a.iter().map(|x| x.to_string()).collect::<Vec<String>>())
    {
        return set.clone();
    }
    // 0 FIRST(a)を空集合に初期化
    let mut first_a = HashSet::new();

    // 以下のステップを，付け加えるものがなくなるまで繰り返す
    loop {
        let before = first_a.clone();
        // 1 aが空系列ならFIRST(a)に空系列を追加
        if a == &["ε"] {
            first_a.insert("ε".to_string());
        }
        // 2 aが終端記号1文字なら，FIRST(a)にその終端記号を追加
        if a.len() == 1 && is_terminal(a[0]) {
            first_a.insert(a[0].to_string());
        }
        // 3 aが非終端記号A1文字なら，Aを左辺にもつすべての生成規則についてのFIRST集合を求め，
        // その要素をFIRST(A)に付け加える
        if a.len() == 1 && !is_terminal(a[0]) {
            let rules = RULES.get().unwrap();
            for r in rules.iter() {
                if r.left == a[0] {
                    let right = r.right.split_whitespace().collect::<Vec<&str>>();
                    let set = first(&right);
                    first_a.extend(set);
                }
            }
        }
        // 4 aが長さ2以上の系列の場合(a=Xbとする)
        if let Some((x, b)) = is_pattern4(a) {
            // 4-1 FIRST(X)がεを含まないなら，FIRST(X)をFIRST(a)に付け加える
            let mut set_x = first(&x);
            if !set_x.contains("ε") {
                first_a.extend(set_x);
            }
            // 4-2 FIRST(X)がεを含むなら，FIRST(X)からεを取り除いたものとFIRST(b)をFIRST(a)に付け加える
            else {
                set_x.remove("ε");
                first_a.extend(set_x);
                let set_b = first(&b);
                first_a.extend(set_b);
            }
        }
        // 5 aが b | y の場合 FIRST(b)とFIRST(y)をFIRST(a)に付け加える
        if let Some((b, y)) = is_pattern5(a) {
            let set_b = first(&b);
            let set_y = first(&y);
            first_a.extend(set_b);
            first_a.extend(set_y);
        }
        // 6 aが{ b } の場合 FIRST(b)とεをFIRST(a)に付け加える
        if let Some(b) = is_pattern6(a) {
            let set_b = first(&b);
            first_a.extend(set_b);
            first_a.insert("ε".to_string());
        }
        // 7 aが[ b ] の場合 FIRST(b)とεをFIRST(a)に付け加える
        if let Some(b) = is_pattern7(a) {
            let set_b = first(&b);
            first_a.extend(set_b);
            first_a.insert("ε".to_string());
        }
        // 8 aが( b ) の場合 FIRST(b)をFIRST(a)に付け加える
        if let Some(b) = is_pattern8(a) {
            let set_b = first(&b);
            first_a.extend(set_b);
        }

        // 付け加えるものがなくなったら終了
        if before == first_a {
            // FIRST(a)を計算済みとして保存
            COMPLETED_FIRST_SET
                .lock()
                .unwrap()
                .insert(a.iter().map(|x| x.to_string()).collect(), first_a.clone());
            break;
        }
    }

    first_a
}

// 2つの系列に分けたい
// 分けられないならNone
// a B
// (a | b) B
// [ a ] B
// { a } B
fn is_pattern4<'a>(a: &[&'a str]) -> Option<(Vec<&'a str>, Vec<&'a str>)> {
    // 長さが1以下なら分けられない
    if a.len() < 2 {
        return None;
    }
    let mut map = HashMap::new();
    map.insert("(", 0);
    map.insert("[", 0);
    map.insert("{", 0);
    let mut left = Vec::new();
    for (i, s) in a.iter().enumerate() {
        // HashMapのvalueがすべて0 かつ1回以上ループを回した状態なら分けられる
        if map.values().all(|x| *x == 0) && i > 0 {
            let right = a[left.len()..].as_ref();
            return Some((left, right.to_vec()));
        }
        match *s {
            "(" => {
                let c = map.get_mut(&"(").unwrap();
                *c += 1;
            }
            ")" => {
                let c = map.get_mut(&"(").unwrap();
                *c -= 1;
            }
            "{" => {
                let c = map.get_mut(&"{").unwrap();
                *c += 1;
            }
            "}" => {
                let c = map.get_mut(&"{").unwrap();
                *c -= 1;
            }
            "[" => {
                let c = map.get_mut(&"[").unwrap();
                *c += 1;
            }
            "]" => {
                let c = map.get_mut(&"[").unwrap();
                *c -= 1;
            }
            _ => {
            }
        }
        left.push(*s);
    }
    None
}

// とにかく，はじめの|の部分とそれ以降の部分に分ける
// そうすれば，あとは再帰的に解決できるか
fn is_pattern5<'a>(a: &[&'a str]) -> Option<(Vec<&'a str>, Vec<&'a str>)> {
    // | が含まれてないならその時点でfalse
    if !a.iter().any(|x| x.contains('|')) {
        return None;
    }
    let mut symbol_count = HashMap::new();
    symbol_count.insert("(", 0);
    symbol_count.insert("[", 0);
    symbol_count.insert("{", 0);

    let mut left = Vec::new();
    for i in a.iter() {
        match *i {
            "(" => {
                let c = symbol_count.get_mut(&"(").unwrap();
                *c += 1;
            }
            ")" => {
                let c = symbol_count.get_mut(&"(").unwrap();
                *c -= 1;
            }
            "{" => {
                let c = symbol_count.get_mut(&"{").unwrap();
                *c += 1;
            }
            "}" => {
                let c = symbol_count.get_mut(&"{").unwrap();
                *c -= 1;
            }
            "[" => {
                let c = symbol_count.get_mut(&"[").unwrap();
                *c += 1;
            }
            "]" => {
                let c = symbol_count.get_mut(&"[").unwrap();
                *c -= 1;
            }
            "|" if symbol_count.values().all(|x| *x == 0) => {
                let right = a[left.len() + 1..].as_ref();
                return Some((left, right.to_vec()));
            }
            _ => {}
        }
        left.push(*i);
    }
    None
}

fn is_pattern6<'a>(a: &[&'a str]) -> Option<Vec<&'a str>> {
    if a.starts_with(&["{"]) && a.ends_with(&["}"]) {
        let buf = a
            .iter()
            .skip(1)
            .take(a.len() - 2).copied()
            .collect::<Vec<&str>>();
        return Some(buf);
    }
    None
}

fn is_pattern7<'a>(a: &[&'a str]) -> Option<Vec<&'a str>> {
    if a.starts_with(&["["]) && a.ends_with(&["]"]) {
        let buf = a
            .iter()
            .skip(1)
            .take(a.len() - 2).copied()
            .collect::<Vec<&str>>();
        return Some(buf);
    }
    None
}

fn is_pattern8<'a>(a: &[&'a str]) -> Option<Vec<&'a str>> {
    if a.starts_with(&["("]) && a.ends_with(&[")"]) {
        let buf = a
            .iter()
            .skip(1)
            .take(a.len() - 2).copied()
            .collect::<Vec<&str>>();
        return Some(buf);
    }
    None
}

// 文字列が終端記号かどうか
// 大文字がないなら終端記号
// キャメルケースは非終端記号
// ( { [ | は非終端記号
fn is_terminal(s: &str) -> bool {
    if s == "(" || s == "{" || s == "[" || s == "|" {
        return false;
    }
    !s.chars().any(|x| x.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first() {
        let rules = [
            Rule {
                left: "E".to_string(),
                right: "T { ( + | - ) } T".to_string(),
            },
            Rule {
                left: "T".to_string(),
                right: "{ ( * | / ) } F".to_string(),
            },
            Rule {
                left: "F".to_string(),
                right: "lp E rp | i | n".to_string(),
            },
        ];
        let first_set = calc_first_set(rules.to_vec());
        let expected = [("E", vec!["lp", "i", "n", "*", "/"]),
            ("T", vec!["lp", "i", "n", "*", "/"]),
            ("F", vec!["lp", "i", "n"])];
        for (k, v) in expected.iter() {
            assert_eq!(
                first_set.get(*k).unwrap(),
                &v.iter().map(|x| x.to_string()).collect()
            );
        }

        // let rules = [
        //     Rule {
        //         left: "E".to_string(),
        //         right: "T { ( + | - ) } T".to_string(),
        //     },
        //     Rule {
        //         left: "T".to_string(),
        //         right: "F { ( * | / ) } F".to_string(),
        //     },
        //     Rule {
        //         left: "F".to_string(),
        //         right: "lp E rp | i | n".to_string(),
        //     },
        // ];
        // let first_set = calc_first_set(rules.to_vec());
        // let expected = [("E", vec!["lp", "i", "n"]),
        //     ("T", vec!["lp", "i", "n"]),
        //     ("F", vec!["lp", "i", "n"])];
        // for (k, v) in expected.iter() {
        //     assert_eq!(
        //         first_set.get(*k).unwrap(),
        //         &v.iter().map(|x| x.to_string()).collect()
        //     );
        // }
    }

    #[test]
    fn test_is_terminal() {
        assert!(is_terminal("a"));
        assert!(!is_terminal("A"));
        assert!(is_terminal("abc"));
        assert!(!is_terminal("Abc"));
        assert!(!is_terminal("["));
        assert!(!is_terminal("{"));
        assert!(!is_terminal("("));
        assert!(!is_terminal("|"));
        assert!(is_terminal("*"));
    }

    #[test]
    fn test_is_pattern4() {
        let a = ["a", "B"];
        let (b, y) = is_pattern4(&a).unwrap();
        assert_eq!(b, ["a"]);
        assert_eq!(y, ["B"]);

        let a = ["(", "a", "|", "b", ")", "B"];
        let (b, y) = is_pattern4(&a).unwrap();
        assert_eq!(b, ["(", "a", "|", "b", ")"]);
        assert_eq!(y, ["B"]);

        let a = ["[", "a", "]", "B"];
        let (b, y) = is_pattern4(&a).unwrap();
        assert_eq!(b, ["[", "a", "]"]);
        assert_eq!(y, ["B"]);

        let a = ["{", "a", "}", "B"];
        let (b, y) = is_pattern4(&a).unwrap();
        assert_eq!(b, ["{", "a", "}"]);
        assert_eq!(y, ["B"]);

        // "( write | writeln ) { '(' OutFor { , OutFor } ')' }"
        let a = ["(", "write", "|", "writeln", ")", "{", "(", "OutFor", "{", ",", "OutFor", "}", ")", "}"];
        let (b, y) = is_pattern4(&a).unwrap();
        assert_eq!(b, ["(", "write", "|", "writeln", ")"]);
        assert_eq!(y, ["{", "(", "OutFor", "{", ",", "OutFor", "}", ")", "}"]);
    }

    #[test]
    fn test_is_pattern5() {
        let a = ["a", "|", "b"];
        let (b, y) = is_pattern5(&a).unwrap();
        assert_eq!(b, ["a"]);
        assert_eq!(y, ["b"]);

        let a = ["(", "a", "|", "b", ")", "B"];
        let r = is_pattern5(&a);
        assert_eq!(r, None);

        let a = ["[", "a", "]", "|", "B"];
        let (b, y) = is_pattern5(&a).unwrap();
        assert_eq!(b, ["[", "a", "]"]);
        assert_eq!(y, ["B"]);

        let a = ["{", "a", "}", "|", "B"];
        let (b, y) = is_pattern5(&a).unwrap();
        assert_eq!(b, ["{", "a", "}"]);
        assert_eq!(y, ["B"]);
    }

    #[test]
    fn test_is_pattern6() {
        let a = ["{", "a", "}"];
        let b = is_pattern6(&a).unwrap();
        assert_eq!(b, ["a"]);

        let a = ["{", "a", "}", "B"];
        let r = is_pattern6(&a);
        assert_eq!(r, None);

        let a = ["{", "a", "|", "B", "}"];
        let r = is_pattern6(&a).unwrap();
        assert_eq!(r, ["a", "|", "B"]);

        let a = ["a", "B"];
        let r = is_pattern6(&a);
        assert_eq!(r, None);
    }

    #[test]
    fn test_is_pattern7() {
        let a = ["[", "a", "]"];
        let b = is_pattern7(&a).unwrap();
        assert_eq!(b, ["a"]);

        let a = ["[", "a", "]", "B"];
        let r = is_pattern7(&a);
        assert_eq!(r, None);

        let a = ["a", "B"];
        let r = is_pattern7(&a);
        assert_eq!(r, None);
    }

    #[test]
    fn test_is_pattern8() {
        let a = ["(", "a", ")"];
        let b = is_pattern8(&a).unwrap();
        assert_eq!(b, ["a"]);

        let a = ["(", "a", ")", "B"];
        let b = is_pattern8(&a);
        assert_eq!(b, None);

        let a = ["a", "B"];
        let r = is_pattern8(&a);
        assert_eq!(r, None);
    }
}
