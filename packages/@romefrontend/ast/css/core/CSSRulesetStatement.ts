import {
	AnyCSSSelector,
	CSSRuleDeclaration,
	NodeBaseWithComments,
} from "../../index";
import {createBuilder} from "../../utils";

// foo {}
export interface CSSRulesetStatement extends NodeBaseWithComments {
	type: "CSSRulesetStatement";
	selectors: Array<AnyCSSSelector>;
	declarations: Array<CSSRuleDeclaration>;
}

export const cssRulesetStatement = createBuilder<CSSRulesetStatement>(
	"CSSRulesetStatement",
	{
		bindingKeys: {},
		visitorKeys: {
			selectors: true,
			declarations: true,
		},
	},
);
