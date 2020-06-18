use crate::modules::account::{
    material::Account,
    tools::{Create, Delete},
};

use crate::modules::account::tests::helper::get_create_member;
use crate::tests::TestContainer;

#[test]
fn issue_delete() {
    let container = TestContainer::new(false);
    let (dns, _node) = container.run();

    let account = Account::with_dns(&dns);
    let post_obj = get_create_member("abc", "abc@abc.de", "password123password123password123");

    let val_pair = account.create(&post_obj.credentials.mail, &post_obj.nickname, &post_obj.credentials.password).unwrap();
    let issue_delete = account.issue_delete(val_pair.member_id);
    assert!(issue_delete.is_ok());
}
