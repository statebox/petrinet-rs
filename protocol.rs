// Auto-generated file using petrinet-rs
trait BothParties<T0,T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13,T14,T15,T16,T17,T18,T19> { 
  fn buy_SWAPLOCK(p0: T2, p1: T15) -> (T3, T16);
  fn publish_SWAPLOCK(p0: T1, p1: T6) -> (T2, T11);
  fn refund_SWAPLOCK(p0: T2, p1: T4) -> (T19);
  fn aliceSignsRefund(p0: T7) -> (T4, T5);
  fn bobSignsLockTx(p0: T5) -> (T6);
  fn bobSendAliceInfo(p0: T8, p1: T9) -> (T7);
  fn bobInit(p0: T10) -> (T8, T9);
  fn claim_REFUND(p0: T19) -> (T3);
  fn spend_REFUND(p0: T19) -> (T1, T18);
  fn publish_XMR_Lock(p0: T13, p1: T11) -> (T12, T14);
  fn Bob_verifyAndSend_Secret(p0: T14) -> (T15);
  fn bob_Claim_XMR(p0: T12, p1: T16) -> (T17);
  fn alice_claim_XMR(p0: T12, p1: T18) -> (T13);
}
