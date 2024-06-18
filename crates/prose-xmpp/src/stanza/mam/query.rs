// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::Jid;
use xmpp_parsers::data_forms::{DataForm, DataFormType, Field, FieldType};
use xmpp_parsers::{mam, rsm};

use crate::ns;
use crate::stanza::message;
use crate::stanza::message::stanza_id;

// https://xmpp.org/extensions/xep-0313.html

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub range: Option<RangeFilter>,
    pub with: Option<Jid>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RangeFilter {
    DateTime(DateTimeFilter),
    /// Requires urn:xmpp:mam:2#extended
    MessageId(MessageIdFilter),
    /// Requires urn:xmpp:mam:2#extended
    Ids(Vec<message::Id>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeFilter {
    Start(DateTime<Utc>),
    End(DateTime<Utc>),
    BetweenInclusive {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageIdFilter {
    Before(message::Id),
    After(message::Id),
    BetweenExclusive {
        before: message::Id,
        after: message::Id,
    },
}

/// RSM does not define the behaviour of including both <before> and <after> in the same request.
/// To retrieve a range of items between two known ids, use before-id and after-id in the query form instead.
#[derive(Debug, Clone, PartialEq)]
pub enum RsmRange {
    /// Use this with a None value to retrieve the last page.
    Before(Option<stanza_id::Id>),
    After(stanza_id::Id),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RsmFilter {
    pub range: Option<RsmRange>,
    pub max: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub filter: Option<Filter>,
    pub rsm_filter: Option<RsmFilter>,
    /// Requires urn:xmpp:mam:2#extended
    pub flip_page: bool,
}

impl Query {
    pub fn into_mam_query(self, query_id: impl Into<String>) -> mam::Query {
        mam::Query {
            queryid: Some(mam::QueryId(query_id.into())),
            node: None,
            form: self.filter.map(Into::into),
            set: self.rsm_filter.map(Into::into),
            flip_page: self.flip_page,
        }
    }
}

impl From<RsmFilter> for rsm::SetQuery {
    fn from(value: RsmFilter) -> Self {
        let mut query = rsm::SetQuery {
            max: value.max,
            after: None,
            before: None,
            index: None,
        };

        match value.range {
            Some(RsmRange::Before(id)) => {
                query.before = Some(id.map(|id| id.into_inner()).unwrap_or(String::new()))
            }
            Some(RsmRange::After(id)) => query.after = Some(id.into_inner()),
            None => {}
        }

        query
    }
}

impl From<Filter> for DataForm {
    fn from(value: Filter) -> Self {
        let mut form = DataForm {
            type_: DataFormType::Submit,
            form_type: Some(ns::MAM.to_string()),
            title: None,
            instructions: None,
            fields: vec![],
        };

        match value.range {
            Some(RangeFilter::DateTime(DateTimeFilter::Start(start))) => form
                .fields
                .push(Field::text_single("start", &start.to_rfc3339())),
            Some(RangeFilter::DateTime(DateTimeFilter::End(end))) => form
                .fields
                .push(Field::text_single("end", &end.to_rfc3339())),
            Some(RangeFilter::DateTime(DateTimeFilter::BetweenInclusive { start, end })) => {
                form.fields
                    .push(Field::text_single("start", &start.to_rfc3339()));
                form.fields
                    .push(Field::text_single("end", &end.to_rfc3339()));
            }
            Some(RangeFilter::MessageId(MessageIdFilter::Before(before))) => form
                .fields
                .push(Field::text_single("before-id", before.as_ref())),
            Some(RangeFilter::MessageId(MessageIdFilter::After(after))) => form
                .fields
                .push(Field::text_single("after-id", after.as_ref())),
            Some(RangeFilter::MessageId(MessageIdFilter::BetweenExclusive { before, after })) => {
                form.fields
                    .push(Field::text_single("before-id", before.as_ref()));
                form.fields
                    .push(Field::text_single("after-id", after.as_ref()));
            }
            Some(RangeFilter::Ids(ids)) => form.fields.push(Field {
                var: Some("ids".to_string()),
                type_: FieldType::TextMulti,
                label: None,
                required: false,
                desc: None,
                options: vec![],
                values: ids.into_iter().map(|id| id.into_inner()).collect(),
                media: vec![],
                validate: None,
            }),
            None => {}
        }

        if let Some(with) = value.with {
            form.fields
                .push(Field::text_single("with", &with.to_string()))
        }

        form
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use insta::assert_snapshot;
    use minidom::Element;

    use super::*;

    #[test]
    fn test_into_mam_query() {
        let mut query = Query {
            filter: None,
            rsm_filter: None,
            flip_page: false,
        };

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"/>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::DateTime(DateTimeFilter::Start(
                Utc.with_ymd_and_hms(2024, 05, 15, 20, 10, 05).unwrap(),
            ))),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="start"><value>2024-05-15T20:10:05+00:00</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::DateTime(DateTimeFilter::End(
                Utc.with_ymd_and_hms(2024, 05, 15, 20, 10, 05).unwrap(),
            ))),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="end"><value>2024-05-15T20:10:05+00:00</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::DateTime(DateTimeFilter::BetweenInclusive {
                start: Utc.with_ymd_and_hms(2024, 05, 15, 20, 10, 05).unwrap(),
                end: Utc.with_ymd_and_hms(2024, 05, 16, 20, 10, 05).unwrap(),
            })),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="start"><value>2024-05-15T20:10:05+00:00</value></field><field var="end"><value>2024-05-16T20:10:05+00:00</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::MessageId(MessageIdFilter::Before(
                "id-1".into(),
            ))),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="before-id"><value>id-1</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::MessageId(MessageIdFilter::After(
                "id-1".into(),
            ))),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="after-id"><value>id-1</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::MessageId(MessageIdFilter::BetweenExclusive {
                before: "id-100".into(),
                after: "id-1".into(),
            })),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="before-id"><value>id-100</value></field><field var="after-id"><value>id-1</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: Some(RangeFilter::Ids(vec!["id-1".into(), "id-2".into()])),
            with: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field type="text-multi" var="ids"><value>id-1</value><value>id-2</value></field></x></query>
        "###);

        query.filter = Some(Filter {
            range: None,
            with: Some("a@prose.org".parse().unwrap()),
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="with"><value>a@prose.org</value></field></x></query>
        "###);

        query.filter = None;
        query.rsm_filter = Some(RsmFilter {
            range: Some(RsmRange::Before(None)),
            max: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><set xmlns='http://jabber.org/protocol/rsm'><before/></set></query>
        "###);

        query.rsm_filter = Some(RsmFilter {
            range: Some(RsmRange::Before(Some("id-1".into()))),
            max: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><set xmlns='http://jabber.org/protocol/rsm'><before>id-1</before></set></query>
        "###);

        query.rsm_filter = Some(RsmFilter {
            range: Some(RsmRange::After("id-1".into())),
            max: None,
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><set xmlns='http://jabber.org/protocol/rsm'><after>id-1</after></set></query>
        "###);

        query.rsm_filter = Some(RsmFilter {
            range: Some(RsmRange::After("id-1".into())),
            max: Some(100),
        });

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><set xmlns='http://jabber.org/protocol/rsm'><max>100</max><after>id-1</after></set></query>
        "###);

        query.rsm_filter = None;
        query.flip_page = true;

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><flip-page/></query>
        "###);

        let query = Query {
            filter: Some(Filter {
                range: Some(RangeFilter::DateTime(DateTimeFilter::Start(
                    Utc.with_ymd_and_hms(2024, 05, 15, 20, 10, 05).unwrap(),
                ))),
                with: Some("user@prose.org".parse().unwrap()),
            }),
            rsm_filter: Some(RsmFilter {
                range: Some(RsmRange::Before(None)),
                max: Some(100),
            }),
            flip_page: true,
        };

        assert_snapshot!(String::from(&Element::from(
            query.clone().into_mam_query("q1")
        )), @r###"
        <query xmlns='urn:xmpp:mam:2' queryid="q1"><x xmlns='jabber:x:data' type="submit"><field type="hidden" var="FORM_TYPE"><value>urn:xmpp:mam:2</value></field><field var="start"><value>2024-05-15T20:10:05+00:00</value></field><field var="with"><value>user@prose.org</value></field></x><set xmlns='http://jabber.org/protocol/rsm'><max>100</max><before/></set><flip-page/></query>
        "###);
    }
}
