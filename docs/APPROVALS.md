# Approvals

Sensitive actions pause the task in `WAITING_FOR_APPROVAL`.

The UI shows:

- Action
- Target
- Data being sent (sanitized)
- Reason
- Risk
- Possible consequences

Buttons: **Approve once**, **Approve for this task**, **Deny**, **Edit**.

The agent cannot approve itself. Denying a step triggers replan or failure,
never a silent skip-and-do-it-anyway.

Purchases (when a connector exists) must show item, seller, price, shipping,
total, payment method, and return information. Until then the action is
denied as unsupported.
