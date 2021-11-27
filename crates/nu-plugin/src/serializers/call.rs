use crate::plugin_capnp::{call, expression};
use nu_protocol::{
    ast::{Call, Expr, Expression},
    ShellError, Span, Spanned, Type,
};

pub(crate) fn serialize_call(call: &Call, mut builder: call::Builder) -> Result<(), ShellError> {
    let mut head = builder.reborrow().init_head();
    head.set_start(call.head.start as u64);
    head.set_end(call.head.end as u64);

    serialize_positional(&call.positional, builder.reborrow());
    serialize_named(&call.named, builder)?;

    Ok(())
}

fn serialize_positional(positional: &[Expression], mut builder: call::Builder) {
    let mut positional_builder = builder.reborrow().init_positional(positional.len() as u32);

    for (index, expression) in positional.iter().enumerate() {
        serialize_expression(expression, positional_builder.reborrow().get(index as u32))
    }
}

fn serialize_named(
    named: &[(Spanned<String>, Option<Expression>)],
    mut builder: call::Builder,
) -> Result<(), ShellError> {
    let mut named_builder = builder
        .reborrow()
        .init_named()
        .init_entries(named.len() as u32);

    for (index, (key, expression)) in named.iter().enumerate() {
        let mut entry_builder = named_builder.reborrow().get(index as u32);
        entry_builder
            .reborrow()
            .set_key(key.item.as_str())
            .map_err(|e| ShellError::InternalError(e.to_string()))?;

        if let Some(expr) = expression {
            let value_builder = entry_builder.init_value();
            serialize_expression(expr, value_builder);
        }
    }

    Ok(())
}

fn serialize_expression(expression: &Expression, mut builder: expression::Builder) {
    match &expression.expr {
        Expr::Garbage => builder.set_garbage(()),
        Expr::Int(val) => builder.set_int(*val),
        Expr::Float(val) => builder.set_float(*val),
        Expr::String(val) => builder.set_string(val),
        _ => {
            // If there is the need to pass other type of argument to the plugin
            // we have to define the encoding for that parameter in this match
        }
    }
}

pub(crate) fn deserialize_call(reader: call::Reader) -> Result<Call, ShellError> {
    let head_reader = reader
        .get_head()
        .map_err(|e| ShellError::InternalError(e.to_string()))?;

    let head = Span {
        start: head_reader.get_start() as usize,
        end: head_reader.get_end() as usize,
    };

    let positional = deserialize_positionals(head, reader)?;
    let named = deserialize_named(head, reader)?;

    Ok(Call {
        decl_id: 0,
        head,
        positional,
        named,
    })
}

fn deserialize_positionals(
    span: Span,
    reader: call::Reader,
) -> Result<Vec<Expression>, ShellError> {
    let positional_reader = reader
        .get_positional()
        .map_err(|e| ShellError::InternalError(e.to_string()))?;

    positional_reader
        .iter()
        .map(|expression_reader| deserialize_expression(span, expression_reader))
        .collect()
}

type NamedList = Vec<(Spanned<String>, Option<Expression>)>;

fn deserialize_named(span: Span, reader: call::Reader) -> Result<NamedList, ShellError> {
    let named_reader = reader
        .get_named()
        .map_err(|e| ShellError::InternalError(e.to_string()))?;

    let entries_list = named_reader
        .get_entries()
        .map_err(|e| ShellError::InternalError(e.to_string()))?;

    let mut entries: Vec<(Spanned<String>, Option<Expression>)> =
        Vec::with_capacity(entries_list.len() as usize);

    for entry_reader in entries_list {
        let item = entry_reader
            .get_key()
            .map_err(|e| ShellError::InternalError(e.to_string()))?
            .to_string();

        let value = if entry_reader.has_value() {
            let value_reader = entry_reader
                .get_value()
                .map_err(|e| ShellError::InternalError(e.to_string()))?;

            let expression = deserialize_expression(span, value_reader)
                .map_err(|e| ShellError::InternalError(e.to_string()))?;

            Some(expression)
        } else {
            None
        };

        let key = Spanned { item, span };

        entries.push((key, value))
    }

    Ok(entries)
}

fn deserialize_expression(
    span: Span,
    reader: expression::Reader,
) -> Result<Expression, ShellError> {
    let expr = match reader.which() {
        Ok(expression::Garbage(())) => Expr::Garbage,
        Ok(expression::Int(val)) => Expr::Int(val),
        Ok(expression::Float(val)) => Expr::Float(val),
        Ok(expression::String(val)) => {
            let string = val
                .map_err(|e| ShellError::InternalError(e.to_string()))?
                .to_string();

            Expr::String(string)
        }
        Err(capnp::NotInSchema(_)) => Expr::Garbage,
    };

    Ok(Expression {
        expr,
        span,
        ty: Type::Unknown,
        custom_completion: None,
    })
}

#[cfg(test)]
mod tests {
    use capnp::serialize;
    use core::panic;

    use super::*;
    use nu_protocol::{
        ast::{Call, Expr, Expression},
        Span, Spanned,
    };

    fn write_buffer(call: &Call, writer: &mut impl std::io::Write) -> Result<(), ShellError> {
        let mut message = ::capnp::message::Builder::new_default();

        let builder = message.init_root::<call::Builder>();
        serialize_call(call, builder)?;

        serialize::write_message(writer, &message)
            .map_err(|e| ShellError::InternalError(e.to_string()))
    }

    fn read_buffer(reader: &mut impl std::io::BufRead) -> Result<Call, ShellError> {
        let message_reader =
            serialize::read_message(reader, ::capnp::message::ReaderOptions::new()).unwrap();

        let reader = message_reader
            .get_root::<call::Reader>()
            .map_err(|e| ShellError::InternalError(e.to_string()))?;

        deserialize_call(reader)
    }

    fn compare_expressions(lhs: &Expression, rhs: &Expression) {
        match (&lhs.expr, &rhs.expr) {
            (Expr::Bool(a), Expr::Bool(b)) => assert_eq!(a, b),
            (Expr::Int(a), Expr::Int(b)) => assert_eq!(a, b),
            (Expr::Float(a), Expr::Float(b)) => assert!((a - b).abs() < f64::EPSILON),
            (Expr::String(a), Expr::String(b)) => assert_eq!(a, b),
            _ => panic!("not matching values"),
        }
    }

    #[test]
    fn call_round_trip() {
        let call = Call {
            decl_id: 1,
            head: Span { start: 0, end: 10 },
            positional: vec![
                Expression {
                    expr: Expr::Float(1.0),
                    span: Span { start: 0, end: 10 },
                    ty: nu_protocol::Type::Float,
                    custom_completion: None,
                },
                Expression {
                    expr: Expr::String("something".into()),
                    span: Span { start: 0, end: 10 },
                    ty: nu_protocol::Type::Float,
                    custom_completion: None,
                },
            ],
            named: vec![
                (
                    Spanned {
                        item: "name".to_string(),
                        span: Span { start: 0, end: 10 },
                    },
                    Some(Expression {
                        expr: Expr::Float(1.0),
                        span: Span { start: 0, end: 10 },
                        ty: nu_protocol::Type::Float,
                        custom_completion: None,
                    }),
                ),
                (
                    Spanned {
                        item: "flag".to_string(),
                        span: Span { start: 0, end: 10 },
                    },
                    None,
                ),
            ],
        };

        let mut buffer: Vec<u8> = Vec::new();
        write_buffer(&call, &mut buffer).expect("unable to serialize message");
        let returned_call = read_buffer(&mut buffer.as_slice()).expect("unable to read buffer");

        assert_eq!(call.head, returned_call.head);
        assert_eq!(call.positional.len(), returned_call.positional.len());

        call.positional
            .iter()
            .zip(returned_call.positional.iter())
            .for_each(|(lhs, rhs)| compare_expressions(lhs, rhs));

        call.named
            .iter()
            .zip(returned_call.named.iter())
            .for_each(|(lhs, rhs)| {
                // Comparing the keys
                assert_eq!(lhs.0.item, rhs.0.item);

                match (&lhs.1, &rhs.1) {
                    (None, None) => {}
                    (Some(a), Some(b)) => compare_expressions(a, b),
                    _ => panic!("not matching values"),
                }
            });
    }
}
